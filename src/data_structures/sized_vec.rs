use crate::{
	lang::{operator::*, size_of, slice_from_raw_parts, slice_from_raw_parts_mut},
	mem::Layout,
	num::Integer,
	prelude::*,
};

/// A [`Vec`] with a custom-sized index type. This allows using index types that
/// are smaller than actual pointers, which can reduce memory usage and be more
/// friendly to CPU caches.
///
/// Using an index type that's larger than [`usize`]
///
/// [`Vec`]: alloc::vec::Vec
pub struct SizedVec<T, S: IndexSize = usize, A: Allocator = GlobalAllocator> {
	capacity: S,
	len: S,
	base_ptr: NonNull<MaybeUninit<T>>,
	alloc: A,
}
impl<T, S: IndexSize> Default for SizedVec<T, S, GlobalAllocator> {
	fn default() -> Self {
		Self::new()
	}
}
impl<T, S: IndexSize> SizedVec<T, S, GlobalAllocator> {
	pub const fn new() -> Self {
		Self::with_allocator(GlobalAllocator)
	}

	pub fn with_capacity(num_items: S) -> Self {
		Self::with_allocator_and_capacity(GlobalAllocator, num_items)
	}
}
impl<T, S: IndexSize, A: Allocator> SizedVec<T, S, A> {
	const BASE_ALLOC_COUNT: S = if size_of::<T>() == 1 {
		S::FIVE + S::THREE
	} else if size_of::<T>() < 1024 {
		S::FOUR
	} else {
		S::ONE
	};

	fn layout(count: S) -> Layout {
		Layout::array::<T>(count.as_usize()).unwrap()
	}

	pub const fn with_allocator(allocator: A) -> Self {
		const { assert!(S::SIZE_BITS <= usize::SIZE_BITS) };
		Self {
			capacity: S::ZERO,
			len: S::ZERO,
			base_ptr: NonNull::dangling(),
			alloc: allocator,
		}
	}
	pub fn with_allocator_and_capacity(allocator: A, num_items: S) -> Self {
		const { assert!(S::SIZE_BITS <= usize::SIZE_BITS) };
		let base_ptr = allocator.allocate(Self::layout(num_items)).unwrap().cast();
		Self {
			capacity: S::ZERO,
			len: S::ZERO,
			base_ptr,
			alloc: allocator,
		}
	}

	pub fn push(&mut self, item: T) -> &mut T {
		if self.len == self.capacity {
			if self.capacity == S::ZERO {
				self.base_ptr = self
					.alloc
					.allocate(Self::layout(Self::BASE_ALLOC_COUNT))
					.unwrap()
					.cast();
			} else if self.capacity == S::MAX {
				panic!("OOM");
			} else {
				self.base_ptr = unsafe {
					self.alloc
						.grow(
							self.base_ptr.cast(),
							Self::layout(self.capacity),
							Self::layout(self.capacity.saturating_mul(S::TWO)),
						)
						.unwrap()
						.cast()
				};
			}
		}

		let ptr = unsafe { &mut *self.base_ptr.add(self.len.as_usize()).as_ptr() };
		self.len += S::ONE;
		ptr.write(item)
	}

	pub fn get(&self, idx: S) -> Option<&T> {
		if idx < self.len {
			unsafe { Some(self.get_unchecked(idx)) }
		} else {
			None
		}
	}
	/// Gets an item from the vector without first verifying that the given
	/// index is in bounds.
	///
	///
	/// # Safety
	///
	/// The caller must ensure the given index is not out-of-bounds of the
	/// vector.
	pub unsafe fn get_unchecked(&self, idx: S) -> &T {
		safety_assert!(idx < self.len);
		unsafe { self.base_ptr.add(idx.as_usize()).as_ref().assume_init_ref() }
	}

	pub fn get_mut(&mut self, idx: S) -> Option<&mut T> {
		if idx < self.len {
			unsafe { Some(self.get_mut_unchecked(idx)) }
		} else {
			None
		}
	}
	/// Mutably gets an item from the vector without first verifying that the
	/// given index is in bounds.
	///
	///
	/// # Safety
	///
	/// The caller must ensure the given index is not out-of-bounds of the
	/// vector.
	pub unsafe fn get_mut_unchecked(&mut self, idx: S) -> &mut T {
		safety_assert!(idx < self.len);
		unsafe { self.base_ptr.add(idx.as_usize()).as_mut().assume_init_mut() }
	}

	pub fn get_range<SO: SizedVecIndexOp<T, S, A>>(&self, range: SO) -> Option<&SO::Output> {
		range.index(self)
	}
	/// # Safety
	///
	/// The caller must verify that the range will not index out-of-bounds.
	pub unsafe fn get_range_unchecked<SO: SizedVecIndexOp<T, S, A>>(
		&self,
		range: SO,
	) -> &SO::Output {
		unsafe { range.index_unchecked(self) }
	}

	pub fn get_range_mut<SO: SizedVecIndexOp<T, S, A>>(
		&mut self,
		range: SO,
	) -> Option<&mut SO::Output> {
		range.index_mut(self)
	}
	/// # Safety
	///
	/// The caller must verify that the range will not index out-of-bounds.
	pub unsafe fn get_range_mut_unchecked<SO: SizedVecIndexOp<T, S, A>>(
		&mut self,
		range: SO,
	) -> &mut SO::Output {
		unsafe { range.index_mut_unchecked(self) }
	}

	pub fn is_empty(&self) -> bool {
		self.len == S::ZERO
	}
	pub fn len(&self) -> S {
		self.len
	}
	pub fn capacity(&self) -> S {
		self.capacity
	}
}
impl<T, S: IndexSize, A: Allocator> Deref for SizedVec<T, S, A> {
	type Target = [T];

	fn deref(&self) -> &Self::Target {
		unsafe { &*slice_from_raw_parts(self.base_ptr.as_ptr().cast(), self.len.as_usize()) }
	}
}
impl<T, S: IndexSize, A: Allocator> DerefMut for SizedVec<T, S, A> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe {
			&mut *slice_from_raw_parts_mut(self.base_ptr.as_ptr().cast(), self.len.as_usize())
		}
	}
}

//
//
// Indexing
//
//

impl<T, S: IndexSize, A: Allocator, SO: SizedVecIndexOp<T, S, A>> Index<SO> for SizedVec<T, S, A> {
	type Output = SO::Output;

	fn index(&self, index: SO) -> &Self::Output {
		index.index(self).unwrap()
	}
}
impl<T, S: IndexSize, A: Allocator, SO: SizedVecIndexOp<T, S, A>> IndexMut<SO>
	for SizedVec<T, S, A>
{
	fn index_mut(&mut self, index: SO) -> &mut Self::Output {
		index.index_mut(self).unwrap()
	}
}

/// Implemented for various-sized types that can index into a [`SizedVec`].
pub trait IndexSize: Integer {
	/// Casts the number to a [`usize`].
	fn as_usize(self) -> usize;
}

/// Implemented for types that can be used in the indexing operation (`[]`) for
/// [`SizedVec`]s.
pub trait SizedVecIndexOp<T, S: IndexSize, A: Allocator> {
	type Output: ?Sized;

	/// Index into the given [`SizedVec`] without first checking if the index
	/// is in-bounds or not.
	///
	///
	/// # Safety
	///
	/// The caller must verify that the index won't read out of bounds.
	unsafe fn index_unchecked(self, vec: &SizedVec<T, S, A>) -> &Self::Output;
	fn index(self, vec: &SizedVec<T, S, A>) -> Option<&Self::Output>;

	/// Mutably index into the given [`SizedVec`] without first checking if the
	/// index is in-bounds or not.
	///
	///
	/// # Safety
	///
	/// The caller must verify that the index won't read out of bounds.
	unsafe fn index_mut_unchecked(self, vec: &mut SizedVec<T, S, A>) -> &mut Self::Output;
	fn index_mut(self, vec: &mut SizedVec<T, S, A>) -> Option<&mut Self::Output>;
}

macro_rules! impl_nums {
	($($ty:ty)*) => {
		$(
			impl IndexSize for $ty {
				fn as_usize(self) -> usize {
					self as usize
				}
			}
			impl<T, A: Allocator> SizedVecIndexOp<T, Self, A> for $ty {
				type Output = T;

				unsafe fn index_unchecked(self, vec: &SizedVec<T, Self, A>) -> &Self::Output {
					unsafe { vec.get_unchecked(self) }
				}
				fn index(self, vec: &SizedVec<T, Self, A>) -> Option<&Self::Output> {
					vec.get(self)
				}

				unsafe fn index_mut_unchecked(self, vec: &mut SizedVec<T, Self, A>) -> &mut Self::Output {
					unsafe { vec.get_mut_unchecked(self) }
				}
				fn index_mut(self, vec: &mut SizedVec<T, Self, A>) -> Option<&mut Self::Output> {
					vec.get_mut(self)
				}
			}
		)*
	};
}
impl_nums!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize);

impl<T, S: IndexSize, A: Allocator> SizedVecIndexOp<T, S, A> for Range<S> {
	type Output = [T];

	unsafe fn index_unchecked(self, vec: &SizedVec<T, S, A>) -> &[T] {
		safety_assert!(self.end < vec.len());

		unsafe {
			&*slice_from_raw_parts(
				vec.base_ptr.add(self.start.as_usize()).as_ptr().cast(),
				(self.end - self.start).as_usize(),
			)
		}
	}
	fn index(self, vec: &SizedVec<T, S, A>) -> Option<&[T]> {
		if self.end < vec.len() {
			Some(unsafe { self.index_unchecked(vec) })
		} else {
			None
		}
	}
	unsafe fn index_mut_unchecked(self, vec: &mut SizedVec<T, S, A>) -> &mut [T] {
		safety_assert!(self.end < vec.len());

		unsafe {
			&mut *slice_from_raw_parts_mut(
				vec.base_ptr.add(self.start.as_usize()).as_ptr().cast(),
				(self.end - self.start).as_usize(),
			)
		}
	}
	fn index_mut(self, vec: &mut SizedVec<T, S, A>) -> Option<&mut [T]> {
		if self.end < vec.len() {
			Some(unsafe { self.index_mut_unchecked(vec) })
		} else {
			None
		}
	}
}
impl<T, S: IndexSize, A: Allocator> SizedVecIndexOp<T, S, A> for RangeInclusive<S> {
	type Output = [T];

	unsafe fn index_unchecked(self, vec: &SizedVec<T, S, A>) -> &[T] {
		safety_assert!(*self.end() < vec.len());

		unsafe {
			&*slice_from_raw_parts(
				vec.base_ptr.add(self.start().as_usize()).as_ptr().cast(),
				(*self.end() - *self.start()).as_usize() + 1,
			)
		}
	}
	fn index(self, vec: &SizedVec<T, S, A>) -> Option<&[T]> {
		if *self.end() < vec.len() {
			Some(unsafe { self.index_unchecked(vec) })
		} else {
			None
		}
	}
	unsafe fn index_mut_unchecked(self, vec: &mut SizedVec<T, S, A>) -> &mut [T] {
		safety_assert!(*self.end() < vec.len());

		unsafe {
			&mut *slice_from_raw_parts_mut(
				vec.base_ptr.add(self.start().as_usize()).as_ptr().cast(),
				(*self.end() - *self.start()).as_usize() + 1,
			)
		}
	}
	fn index_mut(self, vec: &mut SizedVec<T, S, A>) -> Option<&mut [T]> {
		if *self.end() < vec.len() {
			Some(unsafe { self.index_mut_unchecked(vec) })
		} else {
			None
		}
	}
}
impl<T, S: IndexSize, A: Allocator> SizedVecIndexOp<T, S, A> for RangeFrom<S> {
	type Output = [T];

	unsafe fn index_unchecked(self, vec: &SizedVec<T, S, A>) -> &[T] {
		safety_assert!(self.start < vec.len());

		unsafe {
			&*slice_from_raw_parts(
				vec.base_ptr.as_ptr().add(self.start.as_usize()).cast(),
				vec.len().as_usize() - self.start.as_usize(),
			)
		}
	}
	fn index(self, vec: &SizedVec<T, S, A>) -> Option<&[T]> {
		if self.start < vec.len() {
			Some(unsafe { self.index_unchecked(vec) })
		} else {
			None
		}
	}
	unsafe fn index_mut_unchecked(self, vec: &mut SizedVec<T, S, A>) -> &mut [T] {
		safety_assert!(self.start < vec.len());

		unsafe {
			&mut *slice_from_raw_parts_mut(
				vec.base_ptr.as_ptr().add(self.start.as_usize()).cast(),
				vec.len().as_usize() - self.start.as_usize(),
			)
		}
	}
	fn index_mut(self, vec: &mut SizedVec<T, S, A>) -> Option<&mut [T]> {
		if self.start < vec.len() {
			Some(unsafe { self.index_mut_unchecked(vec) })
		} else {
			None
		}
	}
}
impl<T, S: IndexSize, A: Allocator> SizedVecIndexOp<T, S, A> for RangeTo<S> {
	type Output = [T];

	unsafe fn index_unchecked(self, vec: &SizedVec<T, S, A>) -> &[T] {
		safety_assert!(self.end < vec.len());

		unsafe { &*slice_from_raw_parts(vec.base_ptr.as_ptr().cast(), self.end.as_usize()) }
	}
	fn index(self, vec: &SizedVec<T, S, A>) -> Option<&[T]> {
		if self.end < vec.len() {
			Some(unsafe { self.index_unchecked(vec) })
		} else {
			None
		}
	}
	unsafe fn index_mut_unchecked(self, vec: &mut SizedVec<T, S, A>) -> &mut [T] {
		safety_assert!(self.end < vec.len());

		unsafe { &mut *slice_from_raw_parts_mut(vec.base_ptr.as_ptr().cast(), self.end.as_usize()) }
	}
	fn index_mut(self, vec: &mut SizedVec<T, S, A>) -> Option<&mut [T]> {
		if self.end < vec.len() {
			Some(unsafe { self.index_mut_unchecked(vec) })
		} else {
			None
		}
	}
}
impl<T, S: IndexSize, A: Allocator> SizedVecIndexOp<T, S, A> for RangeToInclusive<S> {
	type Output = [T];

	unsafe fn index_unchecked(self, vec: &SizedVec<T, S, A>) -> &[T] {
		safety_assert!(self.end < vec.len());

		unsafe { &*slice_from_raw_parts(vec.base_ptr.as_ptr().cast(), self.end.as_usize() + 1) }
	}
	fn index(self, vec: &SizedVec<T, S, A>) -> Option<&[T]> {
		if self.end < vec.len() {
			Some(unsafe { self.index_unchecked(vec) })
		} else {
			None
		}
	}
	unsafe fn index_mut_unchecked(self, vec: &mut SizedVec<T, S, A>) -> &mut [T] {
		safety_assert!(self.end < vec.len());

		unsafe {
			&mut *slice_from_raw_parts_mut(vec.base_ptr.as_ptr().cast(), self.end.as_usize() + 1)
		}
	}
	fn index_mut(self, vec: &mut SizedVec<T, S, A>) -> Option<&mut [T]> {
		if self.end < vec.len() {
			Some(unsafe { self.index_mut_unchecked(vec) })
		} else {
			None
		}
	}
}
impl<T, S: IndexSize, A: Allocator> SizedVecIndexOp<T, S, A> for RangeFull {
	type Output = [T];

	unsafe fn index_unchecked(self, vec: &SizedVec<T, S, A>) -> &[T] {
		vec
	}
	fn index(self, vec: &SizedVec<T, S, A>) -> Option<&[T]> {
		Some(vec)
	}
	unsafe fn index_mut_unchecked(self, vec: &mut SizedVec<T, S, A>) -> &mut [T] {
		vec
	}
	fn index_mut(self, vec: &mut SizedVec<T, S, A>) -> Option<&mut [T]> {
		Some(vec)
	}
}
