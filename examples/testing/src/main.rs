#![allow(internal_features)]
#![feature(prelude_import)]
#![no_std]
#![no_main]

#[allow(unused_imports)] // why
#[prelude_import]
use crux::prelude::*;

extern crate crux;

#[unsafe(no_mangle)]
fn crux_main() {
	println!("Hello from Crux! 2 + 2 = {}", 2 + 2);
}
