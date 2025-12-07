//! Utilities for dealing with C FFI.

//
//
// Re-exports
//
//

#[doc(inline)]
pub use {
	alloc::ffi::CString,
	core::ffi::{
		CStr, c_char, c_double, c_float, c_int, c_long, c_longlong, c_schar, c_short, c_str,
		c_uchar, c_uint, c_ulong, c_ulonglong, c_ushort, c_void,
	},
	libc::{off_t as c_off_t, size_t as c_size_t, ssize_t as c_ssize_t},
};

//
//
// Custom Utils
//
//

use crate::lang::{slice_from_raw_parts, slice_from_raw_parts_mut};

/// Converts the given pointer to a null-terminated buffer to a byte slice. The
/// slice includes the final null byte.
///
///
/// # Safety
///
/// The pointer must be safe to read and live at least as long as `'a`.
pub unsafe fn null_terminated_pointer_to_slice<'a, const INCLUDE_NULL: bool>(
	ptr: NonNullConst<u8>,
) -> &'a [u8] {
	let slice = unsafe { &*slice_from_raw_parts(ptr.as_ptr(), isize::MAX as usize) };
	let (idx, _) = slice
		.iter()
		.enumerate()
		.find(|(_, byte)| **byte == 0u8)
		.unwrap();

	if INCLUDE_NULL {
		&slice[..=idx]
	} else {
		&slice[..idx]
	}
}
/// Converts the given pointer to a null-terminated buffer to a mutable byte
/// slice. The slice includes the final null byte.
///
///
/// # Safety
///
/// The pointer must be safe to read and live at least as long as `'a`. The
/// pointer must point to a null-terminated buffer.
pub unsafe fn null_terminated_pointer_to_slice_mut<'a, const INCLUDE_NULL: bool>(
	ptr: NonNull<u8>,
) -> &'a mut [u8] {
	let slice =
		unsafe { &mut *slice_from_raw_parts_mut(ptr.as_ptr(), usize::MAX - ptr.as_ptr().addr()) };
	let (idx, _) = slice
		.iter()
		.enumerate()
		.find(|(_, byte)| **byte == 0)
		.unwrap();

	if INCLUDE_NULL {
		&mut slice[..=idx]
	} else {
		&mut slice[..idx]
	}
}
