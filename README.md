# Crux

Crux is a replacement for Rust's standard and core libraries, designed for performant and data-oriented software. It is largely made for a compiler, programming language, and runtime I'm developing (called Shard); in general, though, it's kind of a personalised take on Rust that I may use for my projects.

Note that Crux does not re-implement language primitives (such as `derive`, `Clone`, or `Option`). Instead it re-exports these from Rust's `core` library.

Crux is largely designed to work in the presence of consumer operating systems, so it takes advantage of features like virtual memory and process/thread APIs. Because of this it may not be very friendly to embedded programming. In the future Crux may introduce more feature flags to be more modular, so additional platforms can be supported with a subset of Crux's API.



# Motivation

Rust's standard library is small and somewhat restrictive:
- It lacks some data structures commonly used in data-oriented programming (such as arenas)
- It offers a smaller API for the data structures it does support
	- Its [`HashMap`](https://doc.rust-lang.org/stable/std/collections/struct.HashMap.html) implementation is actually a [re-export of Hashbrown's `HashMap`](https://doc.rust-lang.org/stable/src/std/collections/hash/map.rs.html#4); but the original Hashbrown implementation has a much larger API
- Allocators are largely left as an afterthought
	- The [`Allocator` trait](https://doc.rust-lang.org/stable/std/alloc/trait.Allocator.html) is unstable
	- [`HashMap`](https://doc.rust-lang.org/stable/std/collections/struct.HashMap.html) does not have a customizable allocator, despite [the original implementation in Hashbrown supporting it](https://docs.rs/hashbrown/latest/hashbrown/struct.HashMap.html)

Crux embraces nightly Rust and exposes additional types/functions for performance, fine-grained memory control, direct access to operating system APIs, and data-oriented programming.

In addition to actual reasons for making this library, working with lower-level operating system APIs is teaching me a lot about processes, threads, synchronization, etc.



# Usage

A Crux hello world is more complex than a normal Rust hello world, because the standard library in many cases gets to work directly with the Rust compiler in ways that Crux cannot. So, a hello world will look like this:

```rs
// Needed for the prelude_import feature
#![allow(internal_features)]
// Technically optional, but you'll probably want this - it makes all of the
// commonly-used Rust utilities (println, None, Some, etc) available globally
// without an import
#![feature(prelude_import)]
#![no_std]
// Crux defines the entry point for you to load some things at startup (see
// `crux::rt::entrypoint`). So instead of defining a main function, you'll
// define a `crux_main` function, which Crux will call after setting up.
#![no_main]

// Makes Crux's prelude globally available, so you don't need to explicitly
// import anything from it, just like how `String` and `Box` are automatically
// imported from `std`.
//
// Rust will emit an "unused import" warning here for some reason, regardless
// of if you use the prelude; that's why we add the `allow(unused_imports)`.
#[allow(unused_imports)]
#[prelude_import]
use crux::prelude::*;

// To be honest I don't know why this is needed. All I can tell you is Rust
// sometimes won't detect items from Crux (e.g. a panic handler) without this.
extern crate crux;

// The `crux_main` function Crux's entrypoint will call.
#[unsafe(no_mangle)]
fn crux_main() {
	println!("Hello from Crux! 2 + 2 = {}", 2 + 2);
}
```

Besides the entrypoint to your program clearly being different, Crux should feel fairly similar to `std`. These are the major differences you may run into:
- Crux re-organizes items and submodules to make them easier to find.
	- Crux exposes far fewer top-level modules than `std`. Top-level modules also re-export all items from their submodules.
	- Top-level modules have lots of submodules to still allow for granular modules and global imports.
	- Names are more unique. As an example, `ptr::swap` is renamed to `lang::swap_ptr`, while `mem::swap` is renamed to `lang::swap`.
- The standard library has lots of battle-tested APIs for processes, files, etc. Crux is much newer and therefore is missing a lot of APIs that are in the standard library today.
- On the other hand, Crux has a lot of custom data structures and APIs that you won't find in the standard library (e.g. arena-based allocated types).
- Crux is made to be piercable, and leaves very few implementation details private. You can freely use the same operating system APIs and API abstractions that Crux does, and fully customise parts of its runtime (e.g. the logger).
- Crux has a lot more breaking changes than `std`, since it's newer and I'm still experimenting and figuring out what patterns work and what don't.



# Known Limitations

- Crux requires nightly Rust.
- Crux programs don't print backtraces when they panic.
- Only UNIX operating systems (BSD, Linux, macOS, etc) are supported. Windows support is WIP.
	- In general, Crux is designed to work with operating system APIs, and will likely never work well on embedded systems.



# Crate Features

Crux has the following crate features:
- `default`: `global-os-allocator`, `logging-panic-handler`, `logging`, `term`, `concurrency`
- `global-os-allocator`: Makes Crux's `OsAllocator` type the global allocator.
- `logging-panic-handler`: Provides a default panic handler that simply creates a fatal log and then exits.
- `safety-checks`: Enables additional memory safety assertions in some unsafe functions.
- `logging`: Enables Crux's logging runtime. When disabled, Crux's logging macros do nothing.
- `term`: Enables the `term` module, which provides useful APIs when working with terminals & CLIs
- `concurrency`: Enables the `concurrency` module, which provides APIs for concurrent and parallel code



# Platform Support & Porting to New Platforms

A platform is a processor type (e.g. x86, ARM, RISC-V), operating system (Windows, Linux, macOS), and file type (DLL, ELF, SO). These three together define how a Crux program is loaded and what system APIs it has access to. Therefore Crux must support all three of those parts of a platform to run on a particular computer.

Currently, Crux works when compiled as an ELF executable file to Unix systems on any processor. It is actively tested in executables on x86_64 Linux, so this platform will have the best support.


## Porting to new Operating Systems

Different operating systems use different APIs to accomplish similar (or the same) goal. For example, virtual memory allocations use `mmap` on Unix but `VirtualAlloc` on Windows. Therefore porting Crux to new operating systems mostly just involves finding the new operating system's version of a particular function and calling it over FFI where necessary.

All Crux functions that involve calling OS APIs have this code at the bottom of the function:

```rs
#[cfg(not(supported_os))]
compile_error!("unimplemented on this operating system");
```

This basically gives you a to-do list when compiling for a new operating system. Just find all the compilation errors and replace them with calls to the correct OS API. Then you can add the new OS to the `supported_os` cfg alias in `build.rs`, which will get rid of the `compile_error`s. Finally, run all of Crux's unit tests to make sure everything works as intended.


## Porting to new File Types

Different file types get loaded in different ways by the operating system. As an example, Windows will use the `DllMain` function as the entry point for a `.dll` file, but uses the `mainCRTStartup` (or a few others... it's complicated) function as the entry point for a `.exe` file.

Crux has to load some platform-specific data when the program starts (see `crux::rt::startup_hook`), so it must control the program's entry point


## Porting to new Processors

Currently, Crux does not depend on processor-specific features and works on all processors supported by LLVM (for codegen).

Currently, all of Crux's platform-specific code lies in `crux::os` (Crux's OS APIs) and `crux::rt` (Crux's runtime).



# Reliability

Crux is a low-level library and therefore needs to be quite reliable. Reliable code is largely created with two tools: Static analysis and automated testing. With these two, we can automatically check that Crux handles different variables and scenarios flawlessly, to guarantee reliability.

Static analysis is provided by Rust: The borrow checker, type solver, `clippy`, and `miri`. Crux cannot currently run in `miri` as `miri` does not support allocating virtual memory (which is used by Crux's arena allocator). However, we can still rely on `rustc` and `clippy`. Crux is expected to pass both with no warnings nor errors in every commit.

Testing is written by Crux maintainers. Generally speaking every single Crux API should be tested in a variety of scenarios and any edge case a programmer can think of. We do not want to deal with unexpected and hard-to-trace bugs. Tests also provide a guarantee that changes made by a maintainer do not break code written by any other maintainer. Finally, they can guarantee that we do not re-introduce old bugs. Any time we fix a bug in crux, we must also add a unit test to guarantee the bug does not happen again in the future.

Some tests may require specific feature flags that aren't enabled by default in Crux. These tests can go in the `tests` crate. All other tests should just go at the bottom of the file of the code they're testing, so they're easy to find.


## Reliability Limitations

There are some things that Crux currently can't test nor guarantee through static analysis. For the most part, these are platform differences that are difficult to test due to conditional code compilation or things that fall outside of Rust and can't be tested at all in standard unit tests.

The current list is:
- Different file formats, e.g. run as an ELF or loaded as an SO.
- Various bitnesses, e.g. 32-bit x86 vs 64-bit x86_64.
- Any combination of feature flags (unless we explicitly make two flags uncompatible).

Long-term we should find ways to test all of the above for stronger reliability guarantees.
