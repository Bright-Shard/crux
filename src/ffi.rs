//! Utilities for dealing with C FFI.

//
//
// Re-exports
//
//

#[doc(inline)]
pub use crate::external::{
	alloc::ffi::CString,
	core::ffi::{
		CStr, c_char, c_double, c_float, c_int, c_long, c_longlong, c_schar, c_short, c_str,
		c_uchar, c_uint, c_ulong, c_ulonglong, c_ushort, c_void,
	},
	libc::{off_t as c_off_t, off64_t as c_off64_t, size_t as c_size_t, ssize_t as c_ssize_t},
};
