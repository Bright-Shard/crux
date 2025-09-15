//! Defines the entry point for binary programs written with Crux.

use crate::{
	ffi::*,
	rt::{self, StartupHookInfo},
};

unsafe extern "Rust" {
	safe fn crux_main();
}

#[cfg(unix)]
#[unsafe(no_mangle)]
extern "C" fn main(argc: c_int, argv: *const *const c_char) -> c_int {
	use crate::{io::Writer, os::unix::FileDescriptor};

	let args = unsafe { &*crate::lang::slice_from_raw_parts(argv.cast(), argc as usize) };
	unsafe { rt::startup_hook(StartupHookInfo { args }) };

	crux_main();

	let _ = unsafe { crate::os::unix::FileWriter::new(FileDescriptor::STDOUT).flush() };

	0
}
