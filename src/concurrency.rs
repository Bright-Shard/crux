//! Items for working with concurrent code - code that performs multiple
//! tasks simultaneously.

#[doc(inline)]
pub use {
	alloc::sync::Arc,
	core::sync::atomic::{
		AtomicBool, AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize, AtomicU8, AtomicU16,
		AtomicU32, AtomicU64, AtomicUsize, Ordering as AtomicOrdering,
	},
};
