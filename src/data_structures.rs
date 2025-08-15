//! Structures for storing and organizing data.

pub mod sized_vec;

pub use {
	crate::external::{
		alloc::{
			collections::{
				binary_heap::{self, BinaryHeap},
				btree_map::{self, BTreeMap},
				btree_set::{self, BTreeSet},
			},
			vec::{self, Vec},
		},
		hashbrown::{
			hash_map::{self, HashMap},
			hash_set::{self, HashSet},
			hash_table::{self, HashTable},
		},
	},
	arena_vec::ArenaVec,
	sized_vec::SizedVec,
};

pub mod arena_vec {
	use crate::{
		data_structures::sized_vec::IndexSize,
		lang::UnsafeCell,
		mem::{ArenaAllocator, ArenaPreallocationError, MemoryAmount},
	};

	/// A vector backed by an arena allocator.
	///
	/// Because arenas never move in memory, this vector can be pushed to
	/// immutably; pushing will not move anything in memory and therefore
	/// doesn't need exclusive ownership.
	pub struct ArenaVec<T, S: IndexSize = usize>(UnsafeCell<SizedVec<T, S, ArenaAllocator>>);
	impl<T, S: IndexSize> ArenaVec<T, S> {
		/// Reserve virtual memory for a new arena-backed vector. Errors if
		/// reserving virtual memory fails.
		pub fn new(to_reserve: MemoryAmount) -> Result<Self, ()> {
			Ok(Self(UnsafeCell::new(SizedVec::with_allocator(
				ArenaAllocator::new(to_reserve)?,
			))))
		}
		/// Reserve virtual memory for a new arena-backed vector, then
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
	}
	impl<T, S: IndexSize> From<ArenaAllocator> for ArenaVec<T, S> {
		fn from(value: ArenaAllocator) -> Self {
			Self(UnsafeCell::new(SizedVec::with_allocator(value)))
		}
	}
	impl<T, S: IndexSize> Deref for ArenaVec<T, S> {
		type Target = SizedVec<T, S, ArenaAllocator>;

		fn deref(&self) -> &Self::Target {
			unsafe { &*self.0.get() }
		}
	}
	impl<T, S: IndexSize> DerefMut for ArenaVec<T, S> {
		fn deref_mut(&mut self) -> &mut Self::Target {
			self.0.get_mut()
		}
	}
}
