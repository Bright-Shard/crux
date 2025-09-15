//! Unix API bindings.

use crate::{
	external::libc,
	ffi::*,
	io::Writer,
	lang::{Option, mem::NonNull},
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
	// The `Option<NonNullConst<c_char>>` triggers this. Even though
	// `Option<NonNull<c_char>>` and `Option<*const c_char)` are fine. So
	// presumably a linting mistake.
	#[allow(improper_ctypes)]
	pub unsafe fn getenv(name: NonNullConst<c_char>) -> Option<NonNullConst<c_char>>;
	pub unsafe fn fcntl(fd: FileDescriptor, op: c_int, ...) -> c_int;
	pub safe fn exit(status: c_int) -> !;
}
