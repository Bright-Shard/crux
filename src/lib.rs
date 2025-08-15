#![no_std]
#![no_core]
#![feature(allocator_api)]
#![feature(const_trait_impl)]
#![feature(const_ops)]
#![feature(no_core)]
#![allow(internal_features)] // needed for prelude_import rn
#![feature(prelude_import)]
#![feature(slice_index_methods)]
#![allow(clippy::result_unit_err)]

pub mod data_structures;
pub mod ffi;
pub mod lang;
pub mod mem;
pub mod num;
pub mod os;
pub mod runtime;
pub mod test;

pub use external;

#[allow(unused_imports)] // why
#[prelude_import]
use prelude::*;

pub mod prelude {
	//! Exports most useful types/functions in Crux that are unlikely to collide
	//! with the names of existing types/functions.

	pub use crate::{
		data_structures::{
			ArenaVec, BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, HashTable, SizedVec, Vec,
		},
		lang::{
			Clone, Copy, Default, Deref, DerefMut, Drop, Eq, Err, From, Into, ManuallyDrop,
			MaybeUninit, None, Ok, Option, Ord, PartialEq, PartialOrd, Result, Send, Sized, Some,
			Sync, derive, drop, panic, transmute, transmute_copy,
		},
		mem::{
			AllocError, Allocator, ArenaAllocator, GlobalAllocator, MemoryAmount, NonNull,
			NonNullConst,
		},
		string::{CString, Debug, String},
		test::{
			assert, assert_eq, assert_ne, safety_assert, safety_assert_eq, safety_assert_ne, test,
		},
	};
}

pub mod string {
	//! Functions and types for working with strings.

	pub use external::{
		alloc::{ffi::CString, format, string::String},
		core::{
			ffi::CStr,
			fmt::{Debug, Display},
			format_args,
		},
	};
}
