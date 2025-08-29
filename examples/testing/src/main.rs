#![allow(internal_features)]
#![feature(prelude_import)]

#[allow(unused_imports)] // why
#[prelude_import]
use crux::prelude::*;

fn main() {
	unsafe { crux::rt::startup_hook() };
	println!("Hello from Crux! 2 + 2 = {}", 2 + 2)
}
