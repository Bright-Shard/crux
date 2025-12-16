# Setting up Crux without the Standard Library



## Enable Abort Panics

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



## `main.rs`/`lib.rs`

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
