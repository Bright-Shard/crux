//! Items for working directly with memory and allocations.

use crate::{lang::*, rt::os};

//
//
// Re-exports
//
//

pub use {
	alloc::alloc::Global as GlobalAllocator,
	core::{alloc::GlobalAlloc, prelude::rust_2024::global_allocator},
};

//
//
// Virtual Memory API
//
//

/// Virtual memory that has been reserved, but has not been committed to
/// physical RAM.
///
///
/// # Memory Leaks
///
/// This structure is purely for convenience and does not release reserved
/// memory when dropped. You will leak memory if you do not release it yourself
/// with [`unreserve`].
///
///
/// # What is Reserved Memory?
///
/// Modern operating systems use a virtual memory system that actually relies
/// on two separate kinds of memory: Virtual memory, and physical memory.
/// Virtual memory is what programs interact with via pointers. Physical memory
/// uses a completely different address space and is the actual layout of memory
/// in RAM. When you allocate memory, the OS will allocate that memory in RAM,
/// then create a virtual memory address that maps to the memory it just
/// allocated in RAM. The address in virtual memory can be completely different
/// from the address in RAM.
///
/// Reserved memory refers to memory that exclusively exists in virtual memory
/// and has no actual backing in RAM. Because it doesn't exist in RAM, you can't
/// hold any data in virtual memory - it cannot be written to nor read from.
/// However, you can later commit reserved memory to RAM, which actually
/// allocates the memory in RAM so it can hold data.
///
/// One use case for reserved memory is stable pointers to growable buffers. You
/// can reserve a ridiculous amount of memory (say, 4GB) upfront, without
/// allocating any of it. Then you can slowly commit parts of that 4GB as needed
/// (for example, you could commit 1KB of it initially, but then commit another
/// KB later if the initial buffer fills up). See [`ArenaVec`] for an example of
/// a data structure that does this.
///
///
/// # Safety
///
/// APIs using this structure may safely assume that the region of memory it
/// points to is in fact reserved.
///
/// Creating this structure through safe APIs is safe. Creating this structure
/// through unsafe APIs or by filling the fields manually is unsafe.
///
/// [`ArenaVec`]: crate::data_structures::ArenaVec
#[derive(Clone, Copy)]
pub struct ReservedMemory {
	/// A pointer to the first byte of the reserved memory.
	pub base_ptr: NonNull<()>,
	/// The amount of virtual memory that's reserved.
	pub amount: MemoryAmount,
}
impl ReservedMemory {
	/// Select a specific region of this reserved memory. Errors if the selected
	/// region of memory is outside of this region of reserved memory.
	pub fn select(self, offset: MemoryAmount, len: MemoryAmount) -> Result<Self, ()> {
		let selected_end = unsafe { self.base_ptr.byte_add((offset + len).amount_bytes()) };
		let region_end = unsafe { self.base_ptr.byte_add(self.amount.amount_bytes()) };

		if selected_end <= region_end {
			Ok(unsafe { self.select_unchecked(offset, len) })
		} else {
			Err(())
		}
	}
	/// Select a specific region of this reserved memory.
	///
	///
	/// # Safety
	///
	/// The caller is responsible for ensuring that the selected region of
	/// memory is within this region of reserved memory.
	pub unsafe fn select_unchecked(self, offset: MemoryAmount, len: MemoryAmount) -> Self {
		safety_assert!(
			unsafe { self.base_ptr.byte_add(self.amount.amount_bytes()) }
				> unsafe { self.base_ptr.byte_add((offset + len).amount_bytes()) }
		);

		Self {
			base_ptr: unsafe { self.base_ptr.byte_add(offset.amount_bytes()) },
			amount: len,
		}
	}

	/// Returns all of the memory from `offset` into this reserved memory to the
	/// end of this reserved memory. Errors if the offset goes past the end of
	/// this region of reserved memory.
	pub fn offset(self, offset: MemoryAmount) -> Result<Self, ()> {
		if offset.amount_bytes() < self.amount.amount_bytes() {
			Ok(unsafe { self.offset_unchecked(offset) })
		} else {
			Err(())
		}
	}
	/// Returns all of the memory from `offset` into this reserved memory to the
	/// end of this reserved memory.
	///
	///
	/// # Safety
	///
	/// The caller is responsible for ensuring the offset does not go past the
	/// end of this area of reserved memory.
	pub unsafe fn offset_unchecked(self, offset: MemoryAmount) -> Self {
		safety_assert!(offset.amount_bytes() < self.amount.amount_bytes());

		Self {
			base_ptr: unsafe { self.base_ptr.byte_add(offset.amount_bytes()) },
			amount: self.amount - offset,
		}
	}
}

/// Reserves virtual memory that can later be commited with [`commit`]. See
/// [`ReservedMemory`] for more info. Memory reserved with this function will
/// always be page-aligned.
///
/// Errors if the OS fails to reserve virtual memory.
pub fn reserve(amount: MemoryAmount) -> Result<ReservedMemory, ()> {
	let ptr = {
		#[cfg(unix)]
		{
			let ptr = os::unix::mmap(
				None,
				amount.amount_bytes(),
				libc::PROT_NONE,
				libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
				-1,
				0,
			);
			if ptr == libc::MAP_FAILED {
				return Err(());
			}
			ptr
		}
		#[cfg(windows)]
		{
			os::win32::VirtualAlloc(
				None,
				amount.amount_bytes(),
				os::win32::AllocationType::Reserve as u32,
				os::win32::MemoryProtection::ReadWrite as u32,
			)
		}
		#[cfg(not(supported_os))]
		compile_error!("unimplemented on this operating system");
	};

	Ok(ReservedMemory {
		base_ptr: NonNull::new(ptr.cast()).ok_or(())?,
		amount,
	})
}

/// Commits reserved virtual memory to RAM, effectively allocating the memory
/// and allowing it to be written to/read from.
///
/// `offset` allows you to specify an offset from the originally reserved memory
/// to commit. For example, if you've already commited 1KB of the reserved
/// memory and want to commit the next KB of reserved memory, you'd set `offset`
/// to 1KB.
///
/// Errors if the OS fails to allocate the reserved memory to RAM, or if the
/// `mem` argument doesn't actually point to reserved memory.
pub fn commit(mem: ReservedMemory) -> Result<(), ()> {
	#[cfg(unix)]
	{
		let res = unsafe {
			os::unix::mprotect(
				mem.base_ptr.cast(),
				mem.amount.amount_bytes(),
				libc::PROT_READ | libc::PROT_WRITE,
			)
		};

		if res == 0 { Ok(()) } else { Err(()) }
	}
	#[cfg(windows)]
	{
		let ptr = unsafe {
			os::win32::VirtualAlloc(
				Some(mem.base_ptr.cast()),
				mem.amount.amount_bytes(),
				win32::AllocationType::Commit as u32,
				win32::MemoryProtectionType::ReadWrite as u32,
			)
		};

		if !mem.is_null() { Ok(()) } else { Err(()) }
	}
	#[cfg(not(supported_os))]
	compile_error!("unimplemented on this operating system");
}

/// Releases reserved memory. It is an error to call this function on committed
/// memory.
///
///
/// # Safety
///
/// The memory being released must not be in use. Pointers to this memory are
/// invalid after this function call.
pub unsafe fn unreserve(mem: ReservedMemory) {
	#[cfg(unix)]
	unsafe {
		os::unix::munmap(mem.base_ptr.cast(), mem.amount.amount_bytes());
	}
	#[cfg(windows)]
	unsafe {
		os::win32::VirtualFree(
			mem.base_ptr.cast(),
			0, // must be 0 for FreeType::Release
			win32::FreeType::Release,
		);
	}
	#[cfg(not(supported_os))]
	compile_error!("unimplemented on this operating system");
}

/// Uncommits reserved memory.
///
///
/// # Safety
///
/// The caller must ensure the memory being uncommitted is no longer in use. Any
/// pointers to that memory are invalid after calling this function.
///
/// Note that uncommitting memory may result in entire pages being uncommitted,
/// even if you try to only uncommit a portion of memory in one or more pages.
pub unsafe fn uncommit(mem: ReservedMemory) {
	#[cfg(unix)]
	unsafe {
		os::unix::mprotect(
			mem.base_ptr.cast(),
			mem.amount.amount_bytes(),
			libc::PROT_NONE,
		);
	}
	#[cfg(windows)]
	unsafe {
		os::win32::VirtualFree(
			mem.base_ptr.cast(),
			mem.amount.amount_bytes(),
			win32::FreeType::Decommit,
		);
	}
	#[cfg(not(supported_os))]
	compile_error!("unimplemented on this operating system");
}

/// Allocates read/write memory with standard operating system APIs. Returns an
/// error if the OS fails to allocate.
pub fn allocate(amount: MemoryAmount) -> Result<NonNull<()>, ()> {
	let ptr = {
		#[cfg(unix)]
		{
			let ptr = os::unix::mmap(
				None,
				amount.amount_bytes(),
				libc::PROT_READ | libc::PROT_WRITE,
				libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
				-1,
				0,
			);
			if ptr == libc::MAP_FAILED {
				return Err(());
			}
			let mut t = ptr as usize;
			while t > 0 {
				crate::rt::write_stdout(&[(t % 10) as u8 + 48]);
				t /= 10;
			}
			crate::rt::write_stdout(b"\n");
			ptr
		}
		#[cfg(windows)]
		{
			os::win32::VirtualAlloc(
				None,
				amount.amount_bytes(),
				os::win32::AllocationType::Reserve as u32
					| os::win32::AllocationType::Commit as u32,
				os::win32::MemoryProtection::ReadWrite as u32,
			)
		}
		#[cfg(not(supported_os))]
		compile_error!("unimplemented on this operating system");
	};

	NonNull::new(ptr.cast()).ok_or(())
}
/// Frees allocated memory with standard operating system APIs.
///
/// # Safety
///
/// The memory being freed must not be in use. Pointers to this memory are
/// invalid after this function call.
pub unsafe fn free(ptr: NonNull<()>, amount: MemoryAmount) {
	#[cfg(unix)]
	unsafe {
		os::unix::munmap(ptr.cast(), amount.amount_bytes());
	}
	#[cfg(windows)]
	unsafe {
		os::win32::VirtualFree(
			mem.cast(),
			0, // must be 0 for FreeType::Release
			win32::FreeType::Release,
		);
	}
	#[cfg(not(supported_os))]
	compile_error!("unimplemented on this operating system");
}

//
//
// Allocators
//
//

/// Allocates memory using OS' virtual memory APIs.
///
/// This allocator is *always* available to the program, even before Crux's
/// runtime has been loaded.
#[derive(Clone, Copy, Default, Debug)]
pub struct OsAllocator;
unsafe impl Allocator for OsAllocator {
	fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
		let amount = MemoryAmount::from(layout);
		allocate(amount)
			.map(|ptr| NonNull::slice_from_raw_parts(ptr.cast(), amount.amount_bytes()))
			.map_err(|()| AllocError)
	}
	// No need to separately zero allocated memory on these platforms:
	// - Windows: VirtualAlloc zeroes memory by default
	// - Unix: We use MAP_ANONYMOUS, which zeroes the memory by default
	#[cfg(any(windows, unix))]
	fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
		self.allocate(layout)
	}
	unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
		let amount = MemoryAmount::from(layout);
		unsafe { free(ptr.cast(), amount) }
	}
}
unsafe impl GlobalAlloc for OsAllocator {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		self.allocate(layout).unwrap().as_ptr().cast()
	}
	// Windows: VirtualAlloc zeroes memory by default
	// Unix: Using MAP_ANONYMOUS zeroes the memory by default
	#[cfg(any(windows, unix))]
	unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
		unsafe { self.alloc(layout) }
	}
	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		safety_assert!(!ptr.is_null());
		unsafe { self.deallocate(NonNull::new_unchecked(ptr), layout) };
	}
}

/// Represents a state of used memory in an [`ArenaAllocator`] that the arena
/// can later reset to. Resetting to a checkpoint assumes that any memory
/// allocated after the checkpoint was created is now available to use again,
/// allowing memory re-usage in an arena.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ArenaCheckpoint(MemoryAmount);
impl ArenaCheckpoint {
	/// The amount of memory that was used in the arena when this checkpoint was
	/// created.
	pub fn amount(self) -> MemoryAmount {
		self.0
	}
}

/// Which stage of allocation failed when preallocating an [`ArenaAllocator`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArenaPreallocationError {
	/// Reserving virtual memory failed.
	Reserve,
	/// Committing the preallocated region of virtual memory failed.
	Commit,
	/// The caller tried to preallocate more memory than they reserved.
	PreallocatedMemoryTooLarge,
}

/// Reserves memory from the operating system to create an arena allocator.
/// Arenas are growable buffers that never move in memory.
///
/// See [`ReservedMemory`] for more information about reserved virtual memory
/// and how it allows creating growable buffers that never move.
pub struct VirtualMemoryArena {
	/// Total reserved memory for this arena. Committed memory could (in theory)
	/// use up to this amount of memory.
	pub reserved: ReservedMemory,
	/// The amount of actually usable, committed memory.
	pub committed: Cell<MemoryAmount>,
	/// The amount of committed memory that's been allocated already.
	pub used: Cell<MemoryAmount>,
}
impl VirtualMemoryArena {
	/// Allocate a new arena allocator with the given amount of reserved virtual
	/// memory. Fails if the OS fails to reserve virtual memory.
	pub fn new(to_reserve: MemoryAmount) -> Result<Self, ()> {
		Ok(Self {
			reserved: reserve(to_reserve)?,
			committed: MemoryAmount::ZERO.into(),
			used: MemoryAmount::ZERO.into(),
		})
	}

	/// Allocate a new arena allocator with the given amount of reserved virtual
	/// memory, then preallocate the given amount of memory by committing it.
	///
	/// Fails if the OS fails to reserve virtual memory, or if committing the
	/// preallocated region of that virtual memory fails.
	pub fn new_preallocate(
		to_reserve: MemoryAmount,
		to_commit: MemoryAmount,
	) -> Result<Self, ArenaPreallocationError> {
		let Ok(reserved) = reserve(to_reserve) else {
			return Err(ArenaPreallocationError::Reserve);
		};
		let Ok(to_preallocate) = reserved.select(MemoryAmount::ZERO, to_commit) else {
			return Err(ArenaPreallocationError::PreallocatedMemoryTooLarge);
		};
		let Ok(()) = commit(to_preallocate) else {
			return Err(ArenaPreallocationError::Commit);
		};

		Ok(Self {
			reserved,
			committed: to_commit.into(),
			used: MemoryAmount::ZERO.into(),
		})
	}

	/// Create a "checkpoint" of all the current items in the arena. You can
	/// restore this checkpoint later with [`restore_checkpoint`], which will
	/// (effectively) destroy all items allocated after the checkpoint was
	/// created, allowing you to reuse that memory.
	///
	/// [`restore_checkpoint`]: Self::restore_checkpoint
	pub fn checkpoint(&self) -> ArenaCheckpoint {
		ArenaCheckpoint(self.used.get())
	}
	/// Reset the arena to a checkpoint created previously with [`checkpoint`].
	/// This allows reusing all memory allocated after the checkpoint was
	/// created.
	///
	///
	/// # Safety
	///
	/// The caller is responsible for ensuring there are no valid references to
	/// objects allocated after the checkpoint, as those objects could be
	/// overwritten at any point by future allocations.
	///
	/// The caller is also responsible for making sure objects after the
	/// checkpoint were properly dropped.
	///
	/// [`checkpoint`]: Self::checkpoint
	pub unsafe fn restore_checkpoint(&self, checkpoint: ArenaCheckpoint) {
		self.used.set(checkpoint.0);
	}

	/// "Split" a portion of this arena into a new arena. Future allocations in
	/// this arena will allocate after the split.
	///
	/// This method will fail if you try to split more memory than is available
	/// in the arena.
	pub fn split(&self, amount: MemoryAmount) -> Result<Self, ()> {
		let commited = self.committed.get();
		let used = self.used.get();

		Ok(VirtualMemoryArena {
			reserved: self.reserved.select(used, amount)?,
			committed: Cell::new(commited - used),
			used: Cell::new(MemoryAmount::ZERO),
		})
	}
	/// "Split" a portion of this arena into a new arena. Future allocations in
	/// this arena will allocate after the split.
	///
	///
	/// # Safety
	///
	/// The caller is responsible for ensuring they do not attempt to split more
	/// memory than the arena has left to use.
	pub unsafe fn split_unchecked(&self, amount: MemoryAmount) -> Self {
		let commited = self.committed.get();
		let used = self.used.get();

		safety_assert!(amount < self.available_total_memory());

		VirtualMemoryArena {
			reserved: unsafe { self.reserved.select_unchecked(used, amount) },
			committed: Cell::new(commited - used),
			used: Cell::new(MemoryAmount::ZERO),
		}
	}

	/// Returns the total amount of available memory - regardless of if it's
	/// committed or just reserved - this arena has left.
	pub fn available_total_memory(&self) -> MemoryAmount {
		self.reserved.amount - self.used.get()
	}
	/// Returns the amount of memory this arena has reserved but not committed.
	pub fn available_reserved_memory(&self) -> MemoryAmount {
		self.reserved.amount - self.committed.get()
	}
	/// Returns the amount of committed memory this arena hasn't used yet.
	pub fn available_committed_memory(&self) -> MemoryAmount {
		self.committed.get() - self.used.get()
	}
}
unsafe impl Allocator for VirtualMemoryArena {
	fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
		// aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
		let committed = self.committed.get();
		let used = self.used.get();

		let available = committed - used;
		let needed = MemoryAmount::from(layout);

		if available < needed {
			let diff = needed - available;
			let Ok(to_commit) = self.reserved.select(committed, diff) else {
				return Err(AllocError);
			};
			let Ok(()) = commit(to_commit) else {
				return Err(AllocError);
			};

			self.committed.set(committed + diff);
		}

		let ptr = unsafe {
			NonNull::slice_from_raw_parts(
				self.reserved.base_ptr.byte_add(used.amount_bytes()).cast(),
				needed.amount_bytes(),
			)
		};
		self.used.set(used + needed);

		Ok(ptr)
	}
	// Windows: VirtualAlloc zeroes memory by default
	// Unix: Using MAP_ANONYMOUS zeroes the memory by default
	#[cfg(any(windows, unix))]
	fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
		self.allocate(layout)
	}
	unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {}
}
impl Drop for VirtualMemoryArena {
	fn drop(&mut self) {
		unsafe {
			uncommit(
				self.reserved
					.select_unchecked(MemoryAmount::ZERO, self.committed.get()),
			);
			unreserve(self.reserved);
		}
	}
}

//
//
// Other memory utils
//
//

/// The size of a single page of memory on the current machine.
pub fn page_size() -> usize {
	crate::rt::info().page_size
}

mod memory_amount {
	// declared in a separate module so the `mem` module cannot access
	// `MemoryAmount.0`

	use super::*;

	/// An amount of memory, with convenience initializers for various units of
	/// memory.
	#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
	pub struct MemoryAmount(usize);

	impl MemoryAmount {
		pub const ZERO: Self = Self(0);

		pub const fn bytes(amount: usize) -> Self {
			Self(amount)
		}
		pub const fn kilobytes(amount: usize) -> Self {
			Self(amount.saturating_mul(1000))
		}
		pub const fn kibibytes(amount: usize) -> Self {
			Self(amount.saturating_mul(1024))
		}
		pub const fn megabytes(amount: usize) -> Self {
			Self(amount.saturating_mul(1000 * 1000))
		}
		pub const fn mebibytes(amount: usize) -> Self {
			Self(amount.saturating_mul(1024 * 1024))
		}
		pub const fn gigabytes(amount: usize) -> Self {
			Self(amount.saturating_mul(1000 * 1000 * 1000))
		}
		pub const fn gibibytes(amount: usize) -> Self {
			Self(amount.saturating_mul(1024 * 1024 * 1024))
		}

		pub const fn align_to(self, align: usize) -> Self {
			Self((self.0 as isize + (-(self.0 as isize) & (align as isize - 1))) as usize)
		}
		pub fn page_align(self) -> Self {
			self.align_to(page_size())
		}

		pub const fn amount_bytes(self) -> usize {
			self.0
		}
	}
	impl From<Layout> for MemoryAmount {
		fn from(value: Layout) -> Self {
			Self(value.size())
		}
	}
	impl const Add for MemoryAmount {
		type Output = Self;

		fn add(self, rhs: Self) -> Self::Output {
			Self(self.0 + rhs.0)
		}
	}
	impl AddAssign for MemoryAmount {
		fn add_assign(&mut self, rhs: Self) {
			self.0 += rhs.0
		}
	}
	impl const Sub for MemoryAmount {
		type Output = Self;

		fn sub(self, rhs: Self) -> Self::Output {
			Self(self.0 - rhs.0)
		}
	}
	impl SubAssign for MemoryAmount {
		fn sub_assign(&mut self, rhs: Self) {
			self.0 -= rhs.0
		}
	}
	impl const Mul for MemoryAmount {
		type Output = Self;

		fn mul(self, rhs: Self) -> Self::Output {
			Self(self.0 * rhs.0)
		}
	}
	impl MulAssign for MemoryAmount {
		fn mul_assign(&mut self, rhs: Self) {
			self.0 *= rhs.0
		}
	}
	impl const Div for MemoryAmount {
		type Output = Self;

		fn div(self, rhs: Self) -> Self::Output {
			Self(self.0 / rhs.0)
		}
	}
	impl DivAssign for MemoryAmount {
		fn div_assign(&mut self, rhs: Self) {
			self.0 /= rhs.0
		}
	}
}
pub use memory_amount::MemoryAmount;
