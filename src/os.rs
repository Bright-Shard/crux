//! Items that interact with the operating system, and FFI bindings.

pub use {mem::*, proc::*};

pub mod mem;
pub mod proc;

//
//
// FFI
//
//

#[cfg(unix)]
pub mod unix {
	//! Unix API bindings.

	use crate::{
		external::libc,
		ffi::*,
		io::Writer,
		lang::{Option, ptr::NonNull},
	};

	/// An identifier for a currently open Unix file.
	#[repr(transparent)]
	#[derive(Clone, Copy, PartialEq, Eq, Debug)]
	pub struct FileDescriptor(c_int);
	impl FileDescriptor {
		pub const STDIN: Self = Self(0);
		pub const STDOUT: Self = Self(1);
		pub const STDERR: Self = Self(2);

		/// # Safety
		///
		/// The caller is responsible for ensuring `raw` is a valid file
		/// descriptor.
		pub unsafe fn from_raw(raw: c_int) -> Self {
			Self(raw)
		}
		pub fn as_raw(self) -> c_int {
			self.0
		}
	}

	/// Implements [`Writer`] for the given file descriptor.
	pub struct FileWriter(FileDescriptor);
	impl FileWriter {
		/// Create a writer for the given [`FileDescriptor`].
		///
		///
		/// # Safety
		///
		/// The caller must ensure they have exclusive write access to the given
		/// file descriptor.
		pub unsafe fn new(fd: FileDescriptor) -> Self {
			Self(fd)
		}
	}
	impl Writer for FileWriter {
		type Error = (); // TODO

		fn write(&mut self, bytes: &[u8]) -> Result<usize, Self::Error> {
			let res = unsafe {
				write(
					self.0,
					NonNullConst::from_ref(&bytes[0]).cast(),
					bytes.len() as c_size_t,
				)
			};
			if res == -1 { Err(()) } else { Ok(res as usize) }
		}
		fn flush(&mut self) -> Result<(), Self::Error> {
			let res = unsafe { fsync(self.0) };
			if res == 0 { Ok(()) } else { Err(()) }
		}
	}

	bitset! {
		pub bitset OpenFlags: c_int {
			APPEND = libc::O_APPEND,
			ASYNC = libc::O_ASYNC,
			CLOEXEC = libc::O_CLOEXEC,
			CREAT = libc::O_CREAT,
			DIRECT = libc::O_DIRECT,
			DIRECTORY = libc::O_DIRECTORY,
			DSYNC = libc::O_DSYNC,
			EXCL = libc::O_EXCL,
			LARGEFILE = libc::O_LARGEFILE,
			NOATIME = libc::O_NOATIME,
			NOCTTY = libc::O_NOCTTY,
			NOFOLLOW = libc::O_NOFOLLOW,
			NONBLOCK = libc::O_NONBLOCK,
			NDELAY = libc::O_NDELAY,
			PATH = libc::O_PATH,
			SYNC = libc::O_SYNC,
			TMPFILE = libc::O_TMPFILE,
			TRUNC = libc::O_TRUNC
		}
	}

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
		pub unsafe fn open(path: *const c_char, flags: OpenFlags) -> FileDescriptor;
		pub unsafe fn read(fd: FileDescriptor, buf: NonNull<c_void>, count: c_size_t) -> c_ssize_t;
		pub unsafe fn write(
			fd: FileDescriptor,
			buf: NonNullConst<c_void>,
			count: c_size_t,
		) -> c_ssize_t;
		pub unsafe fn fsync(fd: FileDescriptor) -> c_int;
		pub unsafe fn getenv(name: NonNullConst<c_char>) -> Option<NonNullConst<c_char>>;
		pub unsafe fn fcntl(fd: FileDescriptor, op: c_int, ...) -> c_int;
		pub safe fn exit(status: c_int) -> !;
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
