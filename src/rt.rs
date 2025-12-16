//! A minimal runtime of utilities shipped with every Crux program.
//!
//!
//! # The Crux Runtime
//!
//! Crux's runtime is designed with a few simple goals:
//! 1. Common tasks should be standardised. This allows crates to cohesively
//!    cooperate with shared types, traits, etc.
//! 2. Opt-in overhead. The runtime shouldn't add program overhead by just
//!    existing. If a developer uses a runtime feature it should be clear what
//!    the performance implications are, and the feature shouldn't impact
//!    performance at all when unused.
//!
//! The runtime currently offers the following features:
//! 1. Basic information about the platform the program is running on, such as
//!    the operating system name; see [`CURRENT_OS`] and [`RUNTIME_INFO`].
//! 2. Features Crux was compiled with; see [`SAFETY_CHECKS_ENABLED`] and
//!    [`LOGGING_ENABLED`].
//! 3. Allocation APIs; see [`RuntimeInfo::page_size`] and
//!    [`GLOBAL_OS_ALLOCATOR`].
//! 4. Global program logging; see [`LOGGER`].

pub mod entrypoint;
pub mod hook;
pub mod mem;
pub mod os;
pub mod proc;

#[cfg(target_os = "windows")]
use crate::mem::NonNull;
use crate::{
	ffi::c_void,
	lang::{
		self, Layout, MaybeUninit, cfg,
		mem::{addr_of, addr_of_mut},
		panic,
	},
	logging::{Log, SyncLogger},
};

#[cfg(all(test, feature = "test-harness"))]
pub use test_harness::*;
pub use {entrypoint::*, hook::*, mem::*, os::*, proc::*};

//
//
// Compile-time constants
//
//

/// Operating systems supported by Crux.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Os {
	Linux,
	MacOs,
	Windows,
	UnknownUnix,
}

/// The operating system the program is currently running on.
pub const CURRENT_OS: Os = if cfg!(linux) {
	Os::Linux
} else if cfg!(windows) {
	Os::Windows
} else if cfg!(macos) {
	Os::MacOs
} else if cfg!(unix) {
	Os::UnknownUnix
} else {
	panic!("Crux is not supported on this operating system.")
};

/// If safety checks are enabled in this build.
pub const SAFETY_CHECKS_ENABLED: bool = cfg!(safety_checks);
/// If logging is enabled in this build.
pub const LOGGING_ENABLED: bool = cfg!(logging);

//
//
// Runtime data
//
//

/// Information Crux needs that has to be loaded at runtime. This is stored
/// globally in [`RUNTIME_INFO`] and is loaded by [`startup_hook`].
pub struct RuntimeInfo {
	/// The size of a page of memory on the current machine.
	pub page_size: usize,
	/// The raw CLI args passed to the program at startup.
	pub cli_args_raw: &'static [&'static [u8]],
	/// The CLI args passed to the program at startup, lossily converted to
	/// UTF-8.
	pub cli_args: &'static [&'static str],
}

/// Global instance of [`RuntimeInfo`]. Loaded by [`startup_hook`]. Accessible
/// by [`info`].
pub static mut RUNTIME_INFO: MaybeUninit<RuntimeInfo> = MaybeUninit::uninit();

/// Gets the global [`RuntimeInfo`] instance.
///
/// This function will cause UB if [`startup_hook`] was not called at the
/// program's start. It is assumed that [`startup_hook`] will always be called
/// at the program's start.
pub fn info() -> &'static RuntimeInfo {
	unsafe { (&*addr_of!(RUNTIME_INFO)).assume_init_ref() }
}

#[cfg(feature = "global-os-allocator")]
#[mem::global_allocator]
pub static GLOBAL_OS_ALLOCATOR: mem::OsAllocator = mem::OsAllocator;

#[cfg(all(feature = "logging-panic-handler", feature = "std-compat"))]
compile_error!(
	"Crux: You can't enable the crate feature `logging-panic-handler` and the crate feature `std-compat`. `std` brings its own panic handler, and the logging panic handler would conflict with that."
);
#[cfg_attr(
	all(not(feature = "std-compat"), feature = "logging-panic-handler"),
	panic_handler
)]
pub fn logging_panic_handler(info: &crate::lang::panic::PanicInfo) -> ! {
	crate::logging::fatal!("{}", info);

	#[cfg(supported_os)]
	{
		crate::rt::proc::exit_with_code(101)
	}
	#[cfg(not(supported_os))]
	{
		loop {}
	}
}

//
//
// Logging runtime
//
//

/// The global [`Logger`] instance. Logging macros (e.g. [`log`], [`fatal`])
/// create logs and send them to this logger instance to be handled.
///
/// [`log`]: crate::logging::log
/// [`fatal`]: crate::logging::fatal
pub static mut LOGGER: &'static dyn SyncLogger = &crate::logging::StdoutLogger::default();
/// Sends a log to the global [`LOGGER`] instance.
pub fn emit_log(log: Log) {
	unsafe { &*addr_of_mut!(LOGGER) }.log(log);
}
/// Sets the global [`LOGGER`] instance.
///
///
/// # Safety
///
/// Calling this function updates a global static variable and can therefore
/// lead to race conditions in concurrent code. The caller is responsible for
/// ensuring [`LOGGER`] is not being used by any other code when they call this
/// function.
///
/// The simplest way to use this function safely is to call it one time at
/// startup, and never again.
pub unsafe fn set_logger(mut logger: &'static dyn SyncLogger) {
	let global_logger = unsafe { &mut *addr_of_mut!(LOGGER) };
	lang::mem::swap(&mut logger, global_logger);
	unsafe { lang::mem::drop_in_place(&mut logger) };
}

//
//
// Startup hook
//
//

event! {
	/// An event Crux calls after the binary has been loaded in-memory.
	///
	/// Crux defines two hooks for this event:
	/// - [`rt_startup`]: Loads the Crux runtime.
	/// - [`call_main`]: Calls the `crux_main` function. Only used if the `main`
	///   crate feature is enabled.
	startup,
	fn(StartupHookInfo)
}

/// Information that needs to be passed to [`startup_hook`]. Note that this
/// struct's fields are platform-specific, since different platforms need
/// different data at startup.
#[derive(Clone, Copy)]
pub struct StartupHookInfo {
	/// On Unix, the main function gets `argc` and `argv` parameters, which seem
	/// to be the only way to get the program's CLI arguments. Here we pass
	/// `argv` as a Rust slice.
	#[cfg(unix)]
	pub args: &'static [*const u8],
}

/// A function that must be called at startup by all binaries using Crux. Don't
/// call it yourself unless you know what you're doing.
///
/// Currently, this function just loads the [`RUNTIME_INFO`] global.
///
///
/// # Safety
///
/// This function should only be called one time, at program start, and never
/// again. It's not marked as unsafe so it can conform to the function signature
/// expected by the startup event.
///
/// This function updates global static data and can therefore cause race
/// conditions in concurrent code.
///
/// Many Crux APIs assume this function has been called already when they run,
/// because it loads important OS information used by those APIs. Using Crux
/// APIs before this hook has run may lead to UB.
pub fn startup_hook(info: StartupHookInfo) {
	fn args_to_utf8(raw: &'static [&'static [u8]]) -> &'static [&'static str] {
		let str_len = raw.iter().map(|buf| buf.len()).sum();
		let string: ArenaString<usize> = ArenaString::new_preallocate(
			MemoryAmount::bytes(str_len),
			MemoryAmount::bytes(str_len),
		)
		.unwrap(); // TODO how to handle possible panics during startup?
		let mut out = Vec::with_capacity(Layout::array::<&'static str>(raw.len()).unwrap().size());

		let mut next_base = 0;
		for buf in raw {
			for chunk in buf.utf8_chunks() {
				string.push_str(chunk.valid());
				if !chunk.invalid().is_empty() {
					string.push_char(char::REPLACEMENT_CHARACTER);
				}

				// Safety: string never moves in memory and we leak it at the
				// end of this function. References to it can be static.
				out.push(unsafe { &*(&string[next_base..] as *const str) });
				next_base = string.len();
			}
		}

		lang::forget(string);
		out.leak()
	}

	let runtime_info = {
		#[cfg(target_family = "unix")]
		{
			let page_size = os::unix::sysconf(libc::_SC_PAGE_SIZE) as usize;

			let mut buf = Vec::with_capacity(
				Layout::array::<&'static [u8]>(info.args.len())
					.unwrap()
					.size(),
			);

			for arg in info.args {
				let Some(ptr) = NonNullConst::new(*arg) else {
					continue;
				};
				let slice = unsafe { crate::ffi::null_terminated_pointer_to_slice::<false>(ptr) };
				buf.push(slice);
			}
			let buf: &'static [&'static [u8]] = buf.leak();

			RuntimeInfo {
				page_size,
				cli_args_raw: buf,
				cli_args: args_to_utf8(buf),
			}
		}
		#[cfg(target_os = "windows")]
		{
			let mut sysinfo = MaybeUninit::uninit();
			unsafe { os::win32::GetSystemInfo(NonNull::new_unchecked(sysinfo.as_mut_ptr())) };
			let sysinfo = unsafe { sysinfo.assume_init() };

			RuntimeInfo {
				page_size: sysinfo.page_size as usize,
			}
		}
		#[cfg(not(supported_os))]
		compile_error!("unimplemented on this operating system");
	};

	let global = unsafe { &mut *addr_of_mut!(RUNTIME_INFO) };
	global.write(runtime_info);
}
hook::hook! {
	/// See [`crate::rt::startup_hook`].
	event: crate::events::startup,
	func: startup_hook,
	constraints: []
}

//
//
// Information stored in the binary
//
//

/// Crate types that Crux crates can be compiled as.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
#[repr(u8)]
pub enum CrateType {
	/// The crate is a benchmark that Cargo will run.
	Benchmark = 0,
	/// The crate is being compiled as an executable the user can run.
	Binary = 1,
	/// The crate is being compiled as a dynamic library (`cdylib`) that other
	/// executables can load at runtime.
	Cdylib = 2,
	/// The crate is an example in a library crate.
	Example = 3,
	/// The crate is a set of unit or integration tests.
	Test = 4,
}

// Variables defined in the linker script set by `crux-build`
// Note that you can't get the value of these by reading the static (hence why
// their types are all set to `c_void`). You instead read the value by calling
// `addr_of!(static)`... which will then return a pointer that you can cast to
// a number to get the value.
//
// For example, `__crux_crate_type` is set to a number between 0 and 4 in the
// linker scripts. However, reading `__crux_crate_type` is (afaik) straight up
// UB. Instead, you call `addr_of!(__crux_crate_type)` and cast the resulting
// pointer to a u8, which will then have the number between 0 and 4.
unsafe extern "C" {
	static __crux_ini_start: c_void;
	static __crux_ini_end: c_void;
	static __crux_crate_type: c_void;
}

/// Returns function pointers for all functions that have been registered as ini
/// functions.
///
/// Ini functions are special Crux functions that run before anything else in
/// the Crux application - even before the Crux runtime itself is loaded. They
/// are currently mostly used to implement Crux's event and hook system (see
/// the [`hook`] module).
///
/// Ini functions are *always* unsafe because they run before global variables
/// and the Crux runtime have been initialized. If you want to write code that
/// runs before the `main` function, but after the Crux runtime has loaded, you
/// should look at the [`startup` event] instead.
///
/// [`startup` event]: crate::events::startup
pub fn ini_functions() -> &'static [unsafe fn()] {
	let ini_start = addr_of!(__crux_ini_start) as usize;
	let ini_end = addr_of!(__crux_ini_end) as usize;
	let size = ini_end - ini_start;
	let len = size / (usize::BITS as usize / 8);
	unsafe { &*lang::slice_from_raw_parts(addr_of!(__crux_ini_start) as *const unsafe fn(), len) }
}
/// Returns the [`CrateType`] of the final compiled app Crux is being used in.
pub fn crate_type() -> CrateType {
	let val = addr_of!(__crux_crate_type) as usize as u8;
	unsafe { lang::transmute(val) }
}

/// Register a function as an ini function.
///
/// Usage: `register_ini_function!(function_name);`
///
/// For more information about ini functions, see [`ini_functions`].
#[macro_export]
macro_rules! register_ini_function {
	($func:ident) => {
		#[unsafe(link_section = ".crux.ini")]
		#[used]
		static INI_FUNC: unsafe fn() = $func;
	};
}
pub use crate::register_ini_function;

//
//
// Lazy statics
//
//

#[macro_export]
macro_rules! lazy_static {
	(
		$(#[doc = $doc:literal])*
		$(pub)? static $name:ident: $ty:ty;
		fn load() -> $ty2:ty {
			$($body:tt)*
		}
	) => {
		mod $name {
			use super::*;

			fn load() {
				fn inner() -> $ty2 {
					$($body)*
				}

				unsafe { *$crate::lang::addr_of_mut!($name) = inner() };
			}

			$crate::rt::hook::hook! {
				event: $crate::events::startup,
				func: load,
				constraints: [
					After($crate::hooks::startup_hook),
					Before($crate::hooks::call_main)
				]
			}
		}
		$(#[doc = $doc:literal])*
		$(pub)? static mut $name: $ty = unsafe { $crate::lang::MaybeUninit::uninit().assume_init() };
	};
}

//
//
// Test harness
//
//

/// Provides a harness for running functions decorated with `#[test]` via
/// `cargo test`.
pub mod test_harness {
	crate::rt::event! {
		/// This event is used by the Crux test harness. All tests that should be
		/// run should register with this event.
		///
		/// For convenience, Crux exposes a [`#[test]`] attribute macro. Putting
		/// that on any function will register it with this event.
		///
		/// [`#[test]`]: crux_macros::test
		run_tests, fn()
	}

	/// Runs all tests registered in this Crux binary.
	pub fn run_all_tests() {
		let event = unsafe { run_tests::EVENT.solve() }.expect(
			"Crux CRITICAL ERROR: Failed to solve `run_tests` event, cannot run unit tests",
		);
		for hook in event.as_slice() {
			hook()
		}
	}

	#[cfg(all(feature = "test-harness", test))]
	#[unsafe(no_mangle)]
	fn crux_main() {
		run_all_tests();
	}
}
