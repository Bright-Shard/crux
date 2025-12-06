//! Items that are core and specific to Rust.
//!
//! Nothing in this module is specific to one platform; they're all specific to
//! Rust or how Rust interacts with things like pointers.
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

pub use {compiler::*, essential::*, iter::*, mem::*, op::*, reflect::*, xstat::*};

pub mod op {
	//! Traits that overload operators.

	#[doc(inline)]
	pub use core::{
		cmp::{Eq, Ord, PartialEq, PartialOrd},
		ops::{
			Add, AddAssign, AsyncFn, AsyncFnMut, AsyncFnOnce, BitAnd, BitAndAssign, BitOr,
			BitOrAssign, BitXor, BitXorAssign, Deref, DerefMut, Div, DivAssign, Drop, Fn, FnMut,
			FnOnce, Index, IndexMut, Mul, MulAssign, Range, RangeBounds, RangeFrom, RangeFull,
			RangeInclusive, RangeTo, RangeToInclusive, Sub, SubAssign,
		},
	};
}

pub mod essential {
	//! Items used in pretty much every Rust program.

	#[doc(inline)]
	pub use core::{
		cfg,
		clone::Clone,
		convert::Infallible, // TODO does this belong here?
		convert::{From, Into},
		default::Default,
		matches,
		option::Option::{self, None, Some},
		panic,
		prelude::rust_2024::derive,
		result::Result::{self, Err, Ok},
		todo,
		unreachable,
	};
}

pub mod compiler {
	//! Items that interact with the Rust compiler and type system.

	#[doc(inline)]
	pub use core::{
		any::{Any, TypeId, type_name, type_name_of_val},
		cmp::Ordering,
		column, compile_error, file,
		hint::{assert_unchecked, black_box, select_unpredictable, unreachable_unchecked},
		line,
		marker::{Copy, PhantomData, Send, Sized, Sync, Unpin},
		mem::{
			ManuallyDrop, MaybeUninit, align_of, align_of_val, drop, forget, offset_of, size_of,
			size_of_val, transmute, transmute_copy,
		},
		module_path,
		ptr::{slice_from_raw_parts, slice_from_raw_parts_mut},
	};
}

pub mod mem {
	//! Items for interacting with Rust's memory model - pointers, references,
	//! ownership, allocations, etc.

	#[doc(inline)]
	pub use {
		alloc::borrow::{Cow, ToOwned},
		core::{
			alloc::{AllocError, Allocator, Layout, LayoutError},
			borrow::{Borrow, BorrowMut},
			cell::{self, Cell, LazyCell, RefCell, UnsafeCell},
			mem::{replace, swap, take, zeroed},
			ptr::{
				self, NonNull, addr_of, addr_of_mut, copy, copy_nonoverlapping,
				dangling as dangling_ptr, dangling_mut as dangling_ptr_mut, drop_in_place,
				null as null_ptr, null_mut as null_ptr_mut, read as read_ptr,
				read_unaligned as read_unaligned_ptr, read_volatile as read_volatile_ptr,
				replace as replace_ptr, slice_from_raw_parts, slice_from_raw_parts_mut,
				swap as swap_ptr, swap_nonoverlapping,
			},
		},
	};

	use crate::ffi::{CStr, c_char};

	pub trait AsStatic {
		type Ref: ToOwned + ?Sized;

		#[allow(clippy::wrong_self_convention)]
		fn as_static(self) -> Cow<'static, Self::Ref>;
	}

	impl AsStatic for &&str {
		type Ref = str;
		fn as_static(self) -> Cow<'static, Self::Ref> {
			Cow::Owned(String::from(*self))
		}
	}
	impl AsStatic for &'static str {
		type Ref = str;
		fn as_static(self) -> Cow<'static, Self::Ref> {
			Cow::Borrowed(self)
		}
	}
	impl AsStatic for String {
		type Ref = str;
		fn as_static(self) -> Cow<'static, Self::Ref> {
			Cow::Owned(self)
		}
	}

	/// [`NonNull`], but with a const pointer instead of a mutable pointer.
	#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
	#[repr(transparent)]
	pub struct NonNullConst<T: ?Sized>(*const T);
	impl<T: ?Sized> NonNullConst<T> {
		pub const fn new(ptr: *const T) -> Option<Self> {
			if ptr.is_null() { None } else { Some(Self(ptr)) }
		}
		/// Create a [`NonNullConst`] without checking if the given pointer is
		/// null or not.
		///
		///
		/// # Safety
		///
		/// The caller must guarantee the given pointer is not null
		pub const unsafe fn new_unchecked(ptr: *const T) -> Self {
			Self(ptr)
		}
		pub const fn as_ptr(self) -> *const T {
			self.0
		}
		/// Converts this pointer to a reference.
		///
		///
		/// # Safety
		///
		/// This has all the safety implications of a normal pointer
		/// dereference. The caller is responsible for making sure the pointer
		/// is valid and lives at least as long as the lifetime of the
		/// produced reference.
		pub const unsafe fn as_ref<'a>(self) -> &'a T {
			unsafe { &*self.0 }
		}
		pub const fn from_ref(ref_: &T) -> Self {
			Self(ref_ as *const T)
		}
		pub const fn cast<U>(self) -> NonNullConst<U> {
			NonNullConst(self.0.cast())
		}
		/// Convert this immutable pointer to a mutable pointer.
		///
		///
		/// # Safety
		///
		/// The caller must ensure they can safely mutate the data at this
		/// pointer.
		pub const unsafe fn cast_mut(self) -> NonNull<T> {
			unsafe { NonNull::new_unchecked(self.0.cast_mut()) }
		}
	}

	impl NonNullConst<c_char> {
		/// Convert this [`c_char`] pointer to a [`CStr`].
		///
		///
		/// # Safety
		///
		/// The pointer must be valid for `'a` and must point to a
		/// null-terminated buffer of `c_char`s.
		pub const unsafe fn as_c_str<'a>(self) -> &'a CStr {
			unsafe { CStr::from_ptr(self.as_ptr()) }
		}
	}
}

pub mod iter {
	//! Items for working with iterators.

	pub use core::iter::{Extend, IntoIterator, Iterator};
}

pub mod panic {
	//! Items for dealing with Rust's panicking runtime.

	pub use crate::core::panic::{
		AssertUnwindSafe, Location, PanicInfo, PanicMessage, RefUnwindSafe, UnwindSafe,
	};
}

pub mod xstat {
	//! xstat is a module similar to
	//! [dtolnay's `inventory` crate](https://github.com/dtolnay/inventory)
	//! that allows for cross-crate communication via statics (shortened to
	//! cross-statics, or xstat).

	use crate::lang::UnsafeCell;

	pub struct XStat<T: 'static + ?Sized> {
		pub base: UnsafeCell<Option<&'static XStatEntry<T>>>,
		pub head: UnsafeCell<Option<&'static XStatEntry<T>>>,
	}
	impl<T: 'static + ?Sized> const Default for XStat<T> {
		fn default() -> Self {
			Self {
				base: UnsafeCell::new(None),
				head: UnsafeCell::new(None),
			}
		}
	}
	impl<T: 'static + ?Sized> XStat<T> {
		/// # Safety
		///
		/// This function cannot be called in concurrent contexts.
		pub unsafe fn push(&self, stat: &'static XStatEntry<T>) {
			if let Some(head) = unsafe { &mut *self.head.get() } {
				unsafe { *head.next.get() = Some(stat) };
				*head = stat;
			} else {
				unsafe { *self.base.get() = Some(stat) };
				unsafe { *self.head.get() = Some(stat) };
			}
		}

		/// # Safety
		///
		/// This function cannot be called in concurrent contexts.
		pub unsafe fn entries(&self) -> XStatIter<T> {
			XStatIter {
				node: unsafe { *self.base.get() },
			}
		}
	}
	// yea ts unsafe af tbh
	unsafe impl<T: 'static + ?Sized> Sync for XStat<T> {}
	unsafe impl<T: 'static + ?Sized> Send for XStat<T> {}
	unsafe impl<T: 'static + ?Sized> Sync for XStatEntry<T> {}
	unsafe impl<T: 'static + ?Sized> Send for XStatEntry<T> {}

	/// An iterator over the items in an [`XStat`].
	pub struct XStatIter<T: 'static + ?Sized> {
		node: Option<&'static XStatEntry<T>>,
	}
	impl<T: 'static + ?Sized> Iterator for XStatIter<T> {
		type Item = &'static T;

		fn next(&mut self) -> Option<Self::Item> {
			let node = self.node?;
			if let Some(next_node) = unsafe { &*node.next.get() } {
				self.node = Some(*next_node);
				Some(&node.value)
			} else {
				self.node = None;
				Some(&node.value)
			}
		}
	}

	/// One entry in an [`XStat`].
	pub struct XStatEntry<T: 'static + ?Sized> {
		pub next: UnsafeCell<Option<&'static XStatEntry<T>>>,
		pub value: T,
	}
	impl<T: 'static> XStatEntry<T> {
		pub fn new(value: T) -> Self {
			Self {
				next: UnsafeCell::new(None),
				value,
			}
		}
	}
}

pub mod reflect;
