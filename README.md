# Crux

Crux is a replacement for Rust's standard and core libraries, designed for performant and data-oriented software. It is largely made for a compiler, programming language, and runtime I'm developing (called Shard); in general, though, it's kind of a personalised take on Rust that I may use for my projects.

Note that Crux does not re-implement language primitives (such as `derive`, `Clone`, or `Option`). Instead it re-exports these from Rust's `core` library.

Crux is largely designed to work in the presence of consumer operating systems, so it takes advantage of features like virtual memory and process/thread APIs. Because of this it may not be very friendly to embedded programming. In the future Crux may introduce more feature flags to be more modular, so additional platforms can be supported with a subset of Crux's API.

Currently, Crux supports Unix operating systems, with Windows support being work-in-progress.



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

Libraries just need to add Crux as a dependency.

Binary crates depending on Crux, or using Crux somewhere in their dependency tree, **must** call `crux::runtime::startup_hook` immediately when their program starts. Failure to do so can result in undefined behaviour.

To have Crux's prelude items available by default - like with `std::prelude` - you can use the unstable `prelude_import` feature:
```rs
// lib.rs or main.rs
#![feature(prelude_import)]

#[prelude_import]
use crux::prelude::*;

// some_other_file.rs
type MyTypeAlias = SizedVec<String, u32>; // no imports necessary!
```

Similar to `std`, `core`, and `alloc`, Crux is split into several submodules, though its submodules are structured differently from those crates; generally speaking, Crux prefers less modules and more types/functions in each module, so it's easier to find the piece of code you need. Everything else about Crux can be found in its documentation.



# Crate Features

Crux has the following crate features:
- `default`: No features are enabled by default.
- `safety-checks`: Enables additional memory safety assertions in some unsafe functions.
- `global-os-alloc`: Makes Crux's `OsAllocator` type the global allocator.



# Porting to New Platforms

When compiling for a new operating system, all pieces of code that rely on OS APIs will emit compile-time errors, with code like this:

```rs
#[cfg(not(supported_os))]
compile_error!("unimplemented on this operating system");
```

You'll need to go through and add support for your chosen operating system for each of those functions. Then add the new OS to the `supported_os` cfg alias in `build.rs`, which will get rid of the `compile_error`s. Finally, run all of Crux's unit tests to make sure everything works as intended.

Currently, all of Crux's platform-specific code lies in `crux::os` (Crux's OS APIs) and `crux::rt` (Crux's runtime).
