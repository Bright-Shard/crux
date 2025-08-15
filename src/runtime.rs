//! Data needed by Crux during the program's runtime.

#[cfg(target_os = "windows")]
use crate::mem::NonNull;
use crate::{
	external::libc,
	lang::{MaybeUninit, cfg, panic},
	mem::{addr_of, addr_of_mut},
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
/// program's start.
pub fn info() -> &'static RuntimeInfo {
	unsafe { (&*addr_of!(RUNTIME_INFO)).assume_init_ref() }
}

#[cfg(feature = "global-os-alloc")]
#[crate::mem::global_allocator]
pub static GLOBAL_OS_ALLOCATOR: crate::mem::OsAllocator = crate::mem::OsAllocator;

//
//
// Startup hook
//
//

/// A function that must be called at startup by all binaries using Crux.
/// Not calling this function at startup will lead to undefined behaviour. It
/// should be the first thing a program calls when it launches.
///
/// This function currently just loads the [`RUNTIME_INFO`] global.
pub fn startup_hook() {
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
