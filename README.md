# Crux

Crux is a replacement for Rust's standard and core libraries, designed for performant and data-oriented software. It is largely made for a compiler, programming language, and runtime I'm developing (called Shard); in general, though, it's kind of a personalised take on Rust that I may use for my projects.

Note that Crux does not re-implement language primitives (such as `derive`, `Clone`, or `Option`). Instead it re-exports these from Rust's `core` library.

Crux is largely designed to work in the presence of consumer operating systems, so it takes advantage of features like virtual memory and process/thread APIs. Because of this it may not be very friendly to embedded programming. In the future Crux may introduce more feature flags to be more modular, so additional platforms can be supported with a subset of Crux's API.



# Motivation

Rust's standard library is small and somewhat restrictive:
- It lacks some data structures commonly used in data-oriented programming (such as arenas)
- It does not expose functionality that the standard library itself actually uses to work
- It offers a smaller API for the data structures it does support
	- Its [`HashMap`](https://doc.rust-lang.org/stable/std/collections/struct.HashMap.html) implementation is actually a [re-export of Hashbrown's `HashMap`](https://doc.rust-lang.org/stable/src/std/collections/hash/map.rs.html#4); but the original Hashbrown implementation has a much larger API
- Allocators are largely left as an afterthought
	- The [`Allocator` trait](https://doc.rust-lang.org/stable/std/alloc/trait.Allocator.html) is unstable
	- [`HashMap`](https://doc.rust-lang.org/stable/std/collections/struct.HashMap.html) does not have a customizable allocator, despite [the original implementation in Hashbrown supporting it](https://docs.rs/hashbrown/latest/hashbrown/struct.HashMap.html)

In contrast, Crux focuses on flexibility and performance. It embraces nightly Rust and exposes additional types/functions for performance, fine-grained memory control, direct access to operating system APIs, and data-oriented programming. APIs are intentionally piercable, and provide convenient, high-level wrappers while also allowing you to get your hands dirty with more internal details if you really need to.

I'm also making Crux for educational reasons: Working with lower-level operating system APIs is teaching me a lot about processes, threads, synchronization, etc.



# Usage

The standard library is a special library and gets to take advantage of compiler intrinsics. Crux is not a special library doesn't get those intrinsics. Because of that, Crux is nightly-only, and setup is more involved than for a standard Rust project.


## Setup

All crates that use Crux need to follow these steps.

### Enable Abort Panics

You need to tell Cargo to compile built-in crates from scratch (technical detailsthey have to be compiled without unwinding panics, since unwinding requires `std`). In the root of your project (the folder with the top-level `Cargo.toml` file), create a folder named `.cargo`. In that folder, make a file named `config.toml` with the following contents:

```toml
[unstable]
build-std = [
	"compiler_builtins",
	"core",
	"alloc",
]
panic-abort-tests = true
```

Then, in your `Cargo.toml` file, tell Cargo to use aborting panics instead of unwinding panics:

```toml
[profile.dev] # For debug builds
panic = "abort"

[profile.release] # For release builds
panic = "abort"
```

> Technical Details:
> 
> By default all compiler-provided crates are compiled with unwinding panics. However, unwinding requires the standard library... so Crux cannot use unwinding panics.
>
> These settings force Cargo to compile compiler-provided crates from scratch instead of using the prebuilt versions that were built with unwinding panics. Then setting `panic = "abort"` in `Cargo.toml` makes Cargo compile these crates with aborting panics instead of unwinding panics.

### Add Crux as a dependency

Crux isn't on `crates.io` (the name `crux` is already taken, and it's a pretty common word anyways, so I don't feel like I should take it). So you need to add Crux as a git dependency in `Cargo.toml`, like so:

```toml
TODO
```

Crux has many feature flags. See the bottom of [`Cargo.toml`](./Cargo.toml) for information about what they do.

### Build Script

You need to define a [build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html) that calls a function provided by the [`crux-build` crate](crates/crux-build). First, add a compile-time dependency for `crux-build` in `Cargo.toml`, like so:

```toml
TODO
```

Then you need to call `crux-build::build()` from your build script. Due to Cargo limitations, you need to tell `crux-build` which kinds of Cargo targets are in your Cargo package. Here's an example for a crate that only has a binary:

```rs
use crux_build::CargoTarget;

fn main() {
	crux_build::build(&[CargoTarget::Bin]);
}
```

Or, another example for a binary that also has unit tests:

```rs
use crux_build::CargoTarget;

fn main() {
	crux_build::build(&[CargoTarget::Bin, CargoTarget::Test]);
}
```

If you pass a target type that your package doesn't have, Cargo will emit an error like this:

```
error: invalid instruction `cargo::rustc-link-arg-<something>` from build script of `<your-crate> v0.1.0 (/path/to/crate)`
```

On the other hand, if you don't pass a target type and then try to build that type (e.g. don't pass `CargoTarget::Test` but then run `cargo t`), you'll get a kinda long error that contains text like this:

```
...
note: rust-lld: error: undefined symbol: __crux_crate_type
...
```

You need to pass a `CargoTarget` if you build your crate as a binary, cdylib, build an example of your crate, or run tests or benchmarks on your crate. Yeah, this is annoying. It's entirely Cargo weirdness, I did my best to work around it ergonomically :')

> Technical Details:
> 
> Crux has linker scripts that define several values it uses to determine which kind of binary it's been compiled to (unit test, executable, dynamic library, etc.) and special executable sections used for Crux features. `crux_build::build()` passes additional arguments to Cargo that make it use these linker scripts.
>
> Some of those arguments are for specific Cargo targets (e.g. when a crate is compiled as a cdylib vs when benchmarks are run on it). However, Cargo is very picky about those arguments, and throws a hard error if you try to use them on a package that doesn't have that specific target type.
> 
> Crux shouldn't interfere with your existing build script, if you have one. If it does, please open an issue and let me know - this step isn't meant to conflict with anything.

### `main.rs`/`lib.rs`

You need to enable several unstable features to use Crux:

```rs
// Needed for the prelude_import feature, so you don't need this if you don't
// use prelude imports.
#![allow(internal_features)]
// Technically optional, but you'll probably want this - it's used for the
// prelude import below.
#![feature(prelude_import)]
// If you try to use the standard library, it'll conflict with Crux and error.
#![no_std]
// Crux handles the entrypoint for your crate, not the standard library, so you
// won't be defining a `main` function.
// Instead, if the crate feature `main` is enabled (it is by default), you can
// define a `crux_main` function (like at the bottom of this example file).
#![no_main]

// Makes Crux's prelude globally available, so you don't need to explicitly
// import anything from it, just like how `String`/`None`/`Box`/etc are
// automatically imported from `std`.
//
// Rust will emit an "unused import" warning here for some reason, regardless
// of if you use the prelude; that's why we add the `allow(unused_imports)`.
#[allow(unused_imports)]
#[prelude_import]
use crux::prelude::*;

// To be honest I don't know why this is needed. All I can tell you is Rust
// sometimes won't detect items from Crux (e.g. a panic handler) without this.
extern crate crux;

// Crux's version of the `main` function. Unfortunately Crux can't use `main`
// like normal because that's implemented internally in the compiler; so instead
// it links to this function.
// Crux will only call this function if the crate feature `main` is enabled (it
// is by default).
// 
// Note that, unlike standard Rust, crates compiled as dynamic libraries can
// also have a main function. It'll be called when the library is loaded in
// memory.
#[unsafe(no_mangle)]
fn crux_main() {
	println!("Hello from Crux! 2 + 2 = {}", 2 + 2);
}
```


## General Usage

Besides the above setup steps, Crux should feel fairly similar to `std`. These are the major differences you may run into:
- Crux re-organizes items and submodules to make them easier to find.
	- Crux exposes far fewer top-level modules than `std`. Top-level modules also re-export all items from their submodules.
	- Top-level modules have lots of submodules to still allow for granular modules and global imports.
	- Names are more unique. As an example, `ptr::swap` is renamed to `lang::swap_ptr`, while `mem::swap` is renamed to `lang::swap`.
- The standard library has lots of battle-tested APIs for processes, files, etc. Crux is much newer and therefore is missing a lot of APIs that are in the standard library today.
- On the other hand, Crux has a lot of custom data structures and APIs that you won't find in the standard library (e.g. arena-based allocated types or the hooks API).
- Crux is made to be piercable, and leaves very few implementation details private. You can freely use the same operating system APIs and API abstractions that Crux does, and fully customise parts of its runtime (e.g. the logger).



# Known Limitations

- Crux requires nightly Rust.
- Crux programs don't print backtraces when they panic.
- Only Unix operating systems (BSD, Linux, macOS, etc) are supported. Windows support is WIP.
	- In general, Crux is designed to work with operating system APIs, and will likely never work well on embedded systems.
- Crux will likely have a lot more breaking changes than `std`, since it's newer and I'm still experimenting and figuring out what patterns work and what don't.



# Crate Features

See the docs at the bottom of [`Cargo.toml`](Cargo.toml).



# Platform Support & Porting to New Platforms

A platform is a processor type (e.g. x86, ARM, RISC-V), operating system (Windows, Linux, macOS), and file type (DLL, ELF, SO, etc). These three together define how a Crux program is loaded and what system APIs it has access to. Therefore Crux must support all three of those parts of a platform to run on a particular computer in a particular way.

Currently, Crux works when compiled as an ELF/executable for Unix systems on any processor. It also supports being compiled as a Mach-O file for macOS. On Linux, Crux also works when compiled as a dynamic library/`.so` file.

Crux is actively developed and tested in executables on x86_64 Linux, so this platform will have the best support.


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
