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

The simplest way to use Crux is with `std` compatibility. Crux is able to work with or without Rust's standard library, but Crux with the standard library offers the simplest setup, and lets you use both Crux's APIs and the existing standard library's APIs. This section will walk you through setting up a Crux project with `std` compatibility, but you can see [these docs](./docs/setup-no-std.md) for setting up Crux without the standard library.


### Build Script

You need to define a [build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html) that calls a function provided by the [`crux-build` crate](crates/crux-build). First, add a compile-time dependency for `crux-build` in `Cargo.toml`, like so:

```toml
[build-dependencies]
crux-build = { git = "https://github.com/bright-shard/crux" }
```

> You'll likely want to also specify a commit, like so:
> `crux-build = { git = "...", rev = "<commit hash goes here>" }`
> You can then update the `rev` when a new commit gets pushed to tell Cargo to use a new `crux-build` version.
> 
> Without this Cargo caches the latest version of `crux-build` that it's previously cloned, so even if a new commit gets pushed to `crux-build`, Cargo may use its cache for your local project and not use the updated version of `crux-build`.

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



## Developing with Crux

Besides the above setup steps, Crux should feel fairly similar to `std`. These are the major differences you may run into:
- Crux re-organizes items and submodules to make them easier to find.
	- Crux exposes far fewer top-level modules than `std`. Top-level modules also re-export all items from their submodules.
	- Top-level modules have lots of submodules to still allow for granular modules and global imports.
	- Names are more unique. As an example, `ptr::swap` is renamed to `lang::swap_ptr`, while `mem::swap` is renamed to `lang::swap`. This lets you glob import but still call specific functions.
- The standard library has lots of battle-tested APIs for processes, files, etc. Crux is much newer and therefore is missing a lot of APIs that are in the standard library today.
- On the other hand, Crux has a lot of custom data structures and APIs that you won't find in the standard library (e.g. arena-based allocated types or the hooks API).
- Crux is made to be piercable, and leaves very few implementation details private. You can freely use the same operating system APIs and API abstractions that Crux does, and fully customise parts of its runtime (e.g. the logger).





# Known Limitations

- Crux requires nightly Rust.
- Crux programs that don't use `std-compat` will not print backtraces when they panic.
- Only Linux-like operating systems are supported. macOS and Windows support is WIP.
	- In general, Crux is designed to work with operating system APIs, and will likely never work well on embedded systems.
- Crux is extremely unstable and has lots of breaking changes as I experiment with it.





# Crate Features

See the docs at the bottom of [`Cargo.toml`](Cargo.toml).





# Platform Support

Currently, Crux only supports being compiled as an executable or dynamic library for Linux-like operating systems. See the [platform docs](./docs/platforms.md) for more info.





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





# LLM Code Policy

No.

LLMs are trained to be helpful to the user, disregarding pretty much any other cost, and aren't as reliable as code (since responses are somewhat randomized and largely reliant upon training data).

Crux needs to be reliable and performant, two concepts that are pretty much opposed to LLMs. It's also written in Rust and linker scripts, technologies that LLMs won't have much training data on, so LLMs are more likely to generate bullshit than useful code for a project like this.

If you want to vibe code, go do some JS frontend. This isn't the place for it.
