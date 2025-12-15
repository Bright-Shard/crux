//! Items that interact with the operating system, and FFI bindings to operating
//! system APIs.

//
//
// FFI
//
//

#[cfg(unix)]
pub mod unix;

#[cfg(windows)]
pub mod win32 {
	//! Win32 API bindings.

	use core::{
		ffi::c_void,
		mem::MaybeUninit,
		ptr::{NonNull, addr_of_mut},
	};

	pub static mut SYSTEM_INFO: MaybeUninit<SystemInfo> = MaybeUninit::uninit();
	pub unsafe fn sysinfo() -> &'static mut MaybeUninit<SystemInfo> {
		unsafe { &mut *addr_of_mut!(SYSTEM_INFO) }
	}

	#[repr(C)]
	#[derive(Clone, Copy)]
	struct SystemInfoProcessor {
		pub processor_architecture: u16,
		pub reserved: u16,
	}
	#[repr(C)]
	#[derive(Clone, Copy)]
	union SystemInfoUnion {
		pub oem_id: u32,
		pub system_info: SystemInfoProcessor,
	}

	/// https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/ns-sysinfoapi-system_info
	#[repr(C)]
	struct SystemInfo {
		pub oem: SystemInfoUnion,
		pub page_size: u32,
		pub minimum_application_address: *mut c_void,
		pub maximum_application_address: *mut c_void,
		pub active_processor_mask: usize,
		pub number_of_processors: u32,
		pub processor_type: u32,
		pub allocation_granularity: u32,
		pub processor_level: u16,
		pub processor_revision: u16,
	}

	#[repr(u32)]
	pub enum AllocationType {
		Commit = 0x00001000,
		Reserve = 0x00002000,
	}
	#[repr(u32)]
	pub enum MemoryProtection {
		Execute = 0x10,
		ExecuteRead = 0x20,
		ExecuteReadWrite = 0x40,
		ReadyOnly = 0x02,
		ReadWrite = 0x04,
	}
	#[repr(u32)]
	pub enum FreeType {
		Decommit = 0x00004000,
		Release = 0x00008000,
	}

	#[link(name = "kernel32")]
	unsafe extern "C" {
		pub unsafe fn GetSystemInfo(lpSystemInfo: NonNull<SystemInfo>);
		pub safe fn VirtualAlloc(
			lpAddress: Option<NonNull<c_void>>,
			dwSize: usize,
			flAllocationType: AllocationType,
			flProtect: MemoryProtection,
		) -> Option<NonNull<c_void>>;
		pub unsafe fn VirtualFree(
			lpAddress: NonNull<c_void>,
			dwSize: usize,
			dwFreeType: FreeType,
		) -> bool;
	}
}

// mostly taken from Inventory, with some additional research from myself:
// https://github.com/dtolnay/inventory/blob/master/src/lib.rs
//
// Other useful links:
// - https://github.com/aidansteele/osx-abi-macho-file-format-reference/tree/master
// - https://gist.github.com/x0nu11byt3/bcb35c3de461e5fb66173071a2379779

// /// Execute the given function right when the binary is loaded in memory,
// before /// the main function runs.
// ///
// /// BEWARE! This macro can cause lots of bugs very easily:
// /// - The function passed to the macro must be `extern "C"` (this is not
// checked ///   for you).
// /// - This function runs before any Crux or other runtimes get the chance to
// ///   load. Calling functions that use unloaded runtime data may cause
// undefined ///   behaviour.
// ///
// /// You may prefer using a startup hook that runs after the Crux runtime has
// /// loaded. See [`crate::rt::hook`] and [`crate::events::STARTUP`].
// ///
// /// ```rs
// /// extern "C" fn some_function() {
// ///    do_something();
// /// }
// /// preexec!(some_function);
// /// ```
// #[macro_export]
// macro_rules! preexec {
// 	($func:ident) => {
// 		mod $func {
// 			#[used]
// 			#[cfg_attr(
// 				all(not(target_vendor = "apple"), unix),
// 				unsafe(link_section = ".init_array")
// 			)]
// 			#[cfg_attr(target_vendor = "apple", link_section =
// "__DATA,__mod_init_func")] 			#[cfg_attr(windows, link_section = ".CRT$XCU")]
// 			static PREEXEC: unsafe extern "C" fn() = super::$func;
// 		}
// 	};
// }
// /// Run code when the binary is unloaded from memory, after the main function
// /// exits.
// ///
// /// Functions passed to this macro need to be `extern "C"` (this is not
// checked /// for you).
// ///
// /// You may prefer a Crux startup hook that runs after the `call_main` hook.
// See /// [`crate::rt::hook`], [`crate::events::STARTUP`], and
// /// [`crate::hooks::call_main`].
// ///
// /// ```rs
// /// extern "C" fn some_function() {
// ///    do_something();
// /// }
// /// postexec!(some_function);
// /// ```
// #[macro_export]
// macro_rules! postexec {
// 	($func:ident) => {
// 		mod $func {
// 			#[used]
// 			#[cfg_attr(all(not(target_vendor = "apple"), unix), link_section =
// ".fini_array")] 			#[cfg_attr(target_vendor = "apple", link_section =
// "__DATA,__mod_term_func")] 			#[cfg_attr(windows, link_section)] // todo
// 			static POSTEXEC: unsafe extern "C" fn() = $func;
// 		}
// 	};
// }
// pub use crate::{postexec, preexec};
