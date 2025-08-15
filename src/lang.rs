//! Items that are core to Rust as a language.
//!
//! Examples of items in this module:
//! - Things that interact with the type system (ex: [`TypeId`], [`size_of`])
//! - Basic types used in every Rust program (ex: [`Option`], [`Result`])
//! - Things that communicate with the compiler (ex: [`Drop`], [`black_box`])

//
//
// Re-exports
//
//

pub use {
	compiler::*,
	essential::*,
	external::core::cell::{Cell, LazyCell, RefCell, UnsafeCell},
	operator::*,
};

pub mod operator {
	//! Traits that overload operators.

	pub use external::core::{
		cmp::{Eq, Ord, PartialEq, PartialOrd},
		ops::{
			Add, AddAssign, Deref, DerefMut, Div, DivAssign, Drop, Index, IndexMut, Mul, MulAssign,
			Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
			Sub, SubAssign,
		},
	};
}

pub mod essential {
	//! Items used in pretty much every Rust program.

	pub use external::core::{
		cfg,
		clone::Clone,
		convert::{From, Into},
		default::Default,
		hash::{Hash, Hasher}, // TODO do these belong here?
		option::Option::{self, None, Some},
		panic,
		prelude::rust_2024::derive,
		result::Result::{self, Err, Ok},
		todo,
	};
}

pub mod compiler {
	//! Items that interact with the Rust compiler and type system.

	pub use external::core::{
		compile_error,
		hint::{assert_unchecked, black_box, select_unpredictable, unreachable_unchecked},
		marker::{Copy, PhantomData, Send, Sized, Sync, Unpin},
		mem::{
			ManuallyDrop, MaybeUninit, align_of, align_of_val, drop, forget, offset_of, size_of,
			size_of_val, transmute, transmute_copy,
		},
		ptr::{slice_from_raw_parts, slice_from_raw_parts_mut},
	};
}
