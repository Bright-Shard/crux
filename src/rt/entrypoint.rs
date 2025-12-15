//! Defines the "entry point" for projects compiled with Crux.
//!
//! For executables, this defines the first/"main" function called on app start,
//! which then calls the `crux_main` function.
//!
//! For libraries, this hooks into operating system-specific executable file
//! sections and symbols to execute code as soon as the library gets loaded in
//! memory.

use crate::{
	ffi::*,
	rt::{StartupHookInfo, hook::hook},
};

//
//
// crux_main & binary entrypoint
//
//

#[cfg(feature = "main")]
unsafe extern "Rust" {
	// Main function defined by whatever binary is using crux.
	safe fn crux_main();
}
#[cfg(feature = "std-compat")]
unsafe extern "C" {
	// Main function defined by Rust's standard library.
	#[link_name = "__real_main"]
	safe fn std_main(argc: c_int, argv: *const *const c_char);
}
fn call_main(#[allow(dead_code)] info: StartupHookInfo) {
	#[cfg(feature = "main")]
	crux_main();
	#[cfg(feature = "std-compat")]
	std_main(info.args.len() as _, info.args as *const [*const u8] as _);
}
hook! {
	/// If the crate feature `main` is enabled, calls the user-defined
	/// `crux_main` function.
	/// If the crate feature `std-compat` is enabled, calls the Rust standard
	/// library's main function.
	/// Otherwise does nothing.
	event: crate::events::startup,
	func: call_main,
	constraints: [
		after(crate::hooks::startup_hook),
	]
}

/// Entrypoint for binaries.
#[cfg(unix)]
#[unsafe(no_mangle)]
extern "C" fn __wrap_main(
	argc: c_int,
	argv: *const *const c_char,
	_envp: *const *const c_char,
) -> c_int {
	use crate::{io::Writer, rt::os::unix::FileDescriptor};

	let args = unsafe { &*crate::lang::slice_from_raw_parts(argv.cast(), argc as usize) };
	match entrypoint(StartupHookInfo { args }) {
		Ok(()) => {}
		Err(err) => {
			println!("Crux CRITICAL ERROR: {}", err.error_msg());
			return 1;
		}
	}

	let _ = unsafe { crate::rt::os::unix::FileWriter::new(FileDescriptor::STDOUT).flush() };

	0
}

#[cfg(target_os = "linux")]
mod library_entrypoint {
	use super::*;

	/// Entrypoint for dynamic libraries compiled with Crux on Linux systems.
	#[cfg(unix)]
	extern "C" fn on_library_load() {
		use crate::rt::{self, CrateType};

		if rt::crate_type() == CrateType::Cdylib {
			match entrypoint(StartupHookInfo { args: &[] }) {
				Ok(()) => {}
				Err(err) => {
					println!("Crux CRITICAL ERROR: {}", err.error_msg());
					crate::rt::os::unix::exit(1);
				}
			}
		}
	}

	/// Puts a function pointer to the library entrypoint in the `.init_array`
	/// ELF section. This causes Linux to call the function when the library is
	/// loaded.
	#[used]
	#[unsafe(link_section = ".init_array")]
	static ON_LIBRARY_LOAD: extern "C" fn() = on_library_load;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CruxEntrypointError {
	UnsolvableStartupEvent,
}
impl CruxEntrypointError {
	pub const fn error_msg(self) -> &'static str {
		match self {
			Self::UnsolvableStartupEvent => {
				"The startup event has hooks that conflict with each other, so Crux cannot start running the app."
			}
		}
	}
}

/// Convenience function called by all of Crux's various platform-specific
/// entrypoint functions. They all call this function to help guarantee Crux
/// is setup the correct way.
pub fn entrypoint(info: StartupHookInfo) -> Result<(), CruxEntrypointError> {
	let ini_funcs = crate::rt::ini_functions();
	for func in ini_funcs {
		unsafe { func() };
	}

	let event = unsafe {
		let Ok(to_run) = crate::events::startup::EVENT.solve() else {
			return Err(CruxEntrypointError::UnsolvableStartupEvent);
		};
		to_run
	};
	for hook in event.as_slice() {
		hook(info)
	}

	Ok(())
}
