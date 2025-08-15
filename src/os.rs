//! OS APIs needed by Crux. Currently supports Unix operating systems and
//! Windows.

//
//
// FFI and/or function call caching
//
//

#[cfg(unix)]
pub mod unix {
	//! Unix API bindings.

	use crate::{
		ffi::{c_int, c_long, c_off_t, c_size_t, c_void},
		lang::Option,
		mem::NonNull,
	};

	#[link(name = "c")]
	unsafe extern "C" {
		pub safe fn sysconf(name: i32) -> c_long;
		pub safe fn mmap(
			addr: Option<NonNull<c_void>>,
			length: c_size_t,
			prot: c_int,
			flags: c_int,
			fd: c_int,
			offset: c_off_t,
		) -> *mut c_void;
		pub unsafe fn munmap(addr: NonNull<c_void>, length: c_size_t) -> c_int;
		pub unsafe fn mprotect(addr: NonNull<c_void>, size: c_size_t, prot: c_int) -> c_int;
	}
}

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
