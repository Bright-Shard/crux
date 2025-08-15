#![no_std]

//! External dependencies used by Crux:
//! - Rust's standard `alloc` and `core` crates
//! - Hashbrown, for a hashmap/set/table implementation
//! - Libc, for constants in Unix APIs

pub extern crate alloc;
pub use {core, hashbrown, libc};
