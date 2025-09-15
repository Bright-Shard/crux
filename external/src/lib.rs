#![feature(rustc_private)]
#![no_std]
#![no_main]

//! External dependencies used by Crux:
//! - Rust's standard `alloc` and `core` crates
//! - Hashbrown, for a hashmap/set/table implementation
//! - Libc, for constants in Unix APIs
//!
//! Currently these are all dependencies of std so we just use the `build-std`
//! versions.

pub extern crate alloc;
pub extern crate core;

pub use {hashbrown, libc};
