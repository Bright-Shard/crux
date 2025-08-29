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

#[cfg(target_os = "windows")]
use crate::mem::NonNull;
use crate::{
	external::libc,
	lang::{
		MaybeUninit, cfg, panic,
		ptr::{addr_of, addr_of_mut},
	},
	logging::{EmptyLogger, Log, Logger},
	os,
};

//
//
// Compile-time constants
//
//

/// Operating systems supported by Crux.
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
#[crate::os::mem::global_allocator]
pub static GLOBAL_OS_ALLOCATOR: crate::os::mem::OsAllocator = crate::os::mem::OsAllocator;

#[cfg(all(not(test), feature = "logging-panic-handler"))]
#[panic_handler]
fn panic_handler(info: &crate::lang::panic::PanicInfo) -> ! {
	crate::logging::fatal!("{}", info);

	#[cfg(supported_os)]
	{
		crate::os::proc::exit()
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
pub static mut LOGGER: &'static dyn Logger = &EmptyLogger;
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
pub unsafe fn set_logger(logger: &'static dyn Logger) {
	unsafe {
		let ptr = addr_of_mut!(LOGGER);
		crate::lang::ptr::drop_in_place(ptr);
		*ptr = logger;
	};
}

//
//
// Startup hook
//
//

/// A function that must be called at startup by all binaries using Crux.
/// Not calling this function at startup will lead to undefined behaviour. It
/// should be the first thing a program calls when it launches.
///
/// Currently, this function just loads the [`RUNTIME_INFO`] global.
///
///
/// # Safety
///
/// This function should only be called one time, at program start, and never
/// again.
///
/// It updates global static data and can therefore cause race conditions in
/// concurrent code. Call it before starting any concurrency runtime.
///
/// Many Crux APIs assume this function has been called already when they run,
/// because it loads important OS information used by those APIs. This function
/// should be called as early as possible in the program's startup code.
pub unsafe fn startup_hook() {
	let runtime_info = {
		#[cfg(target_family = "unix")]
		{
			let page_size = os::unix::sysconf(libc::_SC_PAGE_SIZE) as usize;
			RuntimeInfo { page_size }
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
