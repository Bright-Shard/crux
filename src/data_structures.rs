//! Structures for storing and organizing data.

pub mod sized_vec;

pub use self::{
	arena::{ArenaString, ArenaVec},
	binary_heap::BinaryHeap,
	btree_map::BTreeMap,
	btree_set::BTreeSet,
	hash_map::HashMap,
	hash_set::HashSet,
	hash_table::HashTable,
	sized_vec::SizedVec,
	typed_vec::{TypedVec, typed_vec_idx},
	vec::Vec,
};
#[doc(inline)]
pub use crate::external::{
	alloc::{
		boxed::Box,
		collections::{binary_heap, btree_map, btree_set},
		vec,
	},
	hashbrown::{hash_map, hash_set, hash_table},
};

pub mod arena {
	//! Variants of standard allocated data structures that are backed by arena
	//! allocators.

	use crate::{
		data_structures::sized_vec::IndexSize,
		lang::UnsafeCell,
		os::mem::{ArenaAllocator, ArenaPreallocationError, MemoryAmount},
	};

	/// A vector backed by an arena allocator.
	///
	/// Because arenas never move in memory, this vector can be pushed to
	/// immutably; pushing will not move anything in memory and therefore
	/// doesn't need exclusive ownership.
	///
	/// Compared to using an arena by itself, [`ArenaVec`] has two advantages:
	/// 1. It only allows storing one type, which may be nice for some
	///    scenarios.
	/// 2. It calls `drop` on objects in the vec when the vec is dropped. The
	///    standalone arena allocator does not do this.
	pub struct ArenaVec<T, S: const IndexSize = usize>(UnsafeCell<SizedVec<T, S, ArenaAllocator>>);
	impl<T, S: const IndexSize> ArenaVec<T, S> {
		/// Reserve virtual memory for a new arena-backed vector. Errors if
		/// reserving virtual memory fails.
		pub fn new(to_reserve: MemoryAmount) -> Result<Self, ()> {
			Ok(Self(UnsafeCell::new(SizedVec::with_allocator(
				ArenaAllocator::new(to_reserve)?,
			))))
		}
		/// Reserve virtual memory for a new arena-backed vector, then
		/// preallocate some of that memory so it can be used right away.
		pub fn new_preallocate(
			to_reserve: MemoryAmount,
			to_commit: MemoryAmount,
		) -> Result<Self, ArenaPreallocationError> {
			Ok(Self(UnsafeCell::new(SizedVec::with_allocator(
				ArenaAllocator::new_preallocate(to_reserve, to_commit)?,
			))))
		}

		/// Add an item to the end of this arena-backed vector. Because arenas
		/// never move in memory, this can be accomplished with an immutable
		/// reference.
		pub fn push(&self, val: T) {
			unsafe { &mut *self.0.get() }.push(val);
		}
		pub fn extend_slice(&self, slice: &[T]) {
			unsafe { &mut *self.0.get() }.extend_slice(slice);
		}
	}
	impl<T, S: const IndexSize> From<ArenaAllocator> for ArenaVec<T, S> {
		fn from(value: ArenaAllocator) -> Self {
			Self(UnsafeCell::new(SizedVec::with_allocator(value)))
		}
	}
	impl<T, S: const IndexSize> const Deref for ArenaVec<T, S> {
		type Target = SizedVec<T, S, ArenaAllocator>;

		fn deref(&self) -> &Self::Target {
			unsafe { &*self.0.get() }
		}
	}
	impl<T, S: const IndexSize> const DerefMut for ArenaVec<T, S> {
		fn deref_mut(&mut self) -> &mut Self::Target {
			self.0.get_mut()
		}
	}

	/// A [`String`] backed by an arena allocator.
	///
	/// As this string is backed by an arena allocator, it cannot move in
	/// memory, so it can be safely extended with an immutable reference.
	pub struct ArenaString<S: const IndexSize = usize>(ArenaVec<u8, S>);
	impl<S: const IndexSize> ArenaString<S> {
		/// How much memory arena strings reserve when they're created without
		/// explicitly specifying an emount (e.g. via `from`).
		pub const DEFAULT_RESERVE_AMOUNT: MemoryAmount = MemoryAmount::gibibytes(1);

		/// Reserve virtual memory for a new arena-backed vector. Errors if
		/// reserving virtual memory fails.
		pub fn new(to_reserve: MemoryAmount) -> Result<Self, ()> {
			Ok(Self(ArenaVec::new(to_reserve)?))
		}
		/// Reserve virtual memory for a new arena-backed vector, then
		/// preallocate some of that memory so it can be used right away.
		pub fn new_preallocate(
			to_reserve: MemoryAmount,
			to_commit: MemoryAmount,
		) -> Result<Self, ArenaPreallocationError> {
			Ok(Self(ArenaVec::new_preallocate(to_reserve, to_commit)?))
		}

		pub fn push_char(&self, c: char) {
			let mut buf = [0; 4];
			c.encode_utf8(&mut buf);
			self.0.extend_slice(&buf);
		}
		pub fn push_str(&self, s: &str) {
			self.0.extend_slice(s.as_bytes());
		}

		pub const fn as_str(&self) -> &str {
			unsafe { str::from_utf8_unchecked(&self.0) }
		}
	}
	impl<S: const IndexSize> From<&str> for ArenaString<S> {
		fn from(value: &str) -> Self {
			let this = Self::new_preallocate(
				Self::DEFAULT_RESERVE_AMOUNT,
				MemoryAmount::bytes(value.len()),
			)
			.unwrap();
			this.push_str(value);
			this
		}
	}
	impl<S: const IndexSize> Deref for ArenaString<S> {
		type Target = str;

		fn deref(&self) -> &Self::Target {
			self.as_str()
		}
	}
	impl<S: const IndexSize> DerefMut for ArenaString<S> {
		fn deref_mut(&mut self) -> &mut Self::Target {
			unsafe { str::from_utf8_unchecked_mut(&mut self.0) }
		}
	}
}

pub mod typed_vec {
	use crate::data_structures::sized_vec::IndexSize;

	pub trait TypedVecIndex: Clone + Copy {
		type Index: const IndexSize;

		fn raw(self) -> Self::Index;
		unsafe fn from_raw(raw: Self::Index) -> Self;
	}

	pub struct TypedVec<T, S: TypedVecIndex, A: Allocator = GlobalAllocator>(
		SizedVec<T, S::Index, A>,
	);
	impl<T, S: TypedVecIndex> Default for TypedVec<T, S, GlobalAllocator> {
		fn default() -> Self {
			Self::new()
		}
	}
	impl<T, S: TypedVecIndex> TypedVec<T, S, GlobalAllocator> {
		pub const fn new() -> Self {
			Self(SizedVec::new())
		}
		pub fn with_capacity(num_items: S::Index) -> Self {
			Self(SizedVec::with_capacity(num_items))
		}
	}
	impl<T, S: TypedVecIndex, A: Allocator> TypedVec<T, S, A> {
		pub const fn with_allocator(allocator: A) -> Self {
			Self(SizedVec::with_allocator(allocator))
		}
		pub fn with_allocator_and_capacity(allocator: A, num_items: S::Index) -> Self {
			Self(SizedVec::with_allocator_and_capacity(allocator, num_items))
		}

		pub fn get(&self, idx: S) -> Option<&T> {
			self.0.get(idx.raw())
		}
		pub fn get_mut(&mut self, idx: S) -> Option<&mut T> {
			self.0.get_mut(idx.raw())
		}

		pub fn push(&mut self, item: T) -> &mut T {
			self.0.push(item)
		}
	}

	#[macro_export]
	macro_rules! typed_vec_idx {
		($($ty:ident: $size:ty),*) => {
			$(
			#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
			pub struct $ty($size);
			impl $crate::data_structures::typed_vec::TypedVecIndex for $ty {
				type Index = $size;

				fn raw(self) -> Self::Index {
					self.0
				}
				unsafe fn from_raw(raw: Self::Index) -> Self {
					Self(raw)
				}
			}
			)*
		};
	}
	pub use crate::typed_vec_idx;
}
