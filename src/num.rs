//! Utilities for working with numbers.

use crate::{
	lang::{Add, AddAssign, Copy, Div, DivAssign, Mul, MulAssign, Ord, Sized, Sub, SubAssign},
	text::{Debug, Display},
};

#[doc(inline)]
pub use crate::external::core::num::NonZero;

/// Abstracts over all of Rust's integer types, allowing you to create
/// functions that accept arbitrarily-sized integers.
#[rustfmt::skip]
pub const trait Integer:
	Sized
	+ Ord
	+ Copy
	+ Debug
	+ Display
	+ const Add<Output = Self>
	+ AddAssign
	+ const Sub<Output = Self>
	+ SubAssign
	+ const Mul<Output = Self>
	+ MulAssign
	+ const Div<Output = Self>
	+ DivAssign
{
	const MAX: Self;
	const MIN: Self;
	const SIZE_BITS: u8;
	const SIZE_BYTES: u8;
	const SIGNED: bool;

	const ZERO: Self;
	const ONE: Self;
	const TWO: Self;
	const THREE: Self;
	const FOUR: Self;
	const FIVE: Self;

	fn saturating_add(self, rhs: Self) -> Self;
	fn saturating_sub(self, rhs: Self) -> Self;
	fn saturating_div(self, rhs: Self) -> Self;
	fn saturating_mul(self, rhs: Self) -> Self;

	fn checked_add(self, rhs: Self) -> Option<Self>;
	fn checked_sub(self, rhs: Self) -> Option<Self>;
	fn checked_div(self, rhs: Self) -> Option<Self>;
	fn checked_mul(self, rhs: Self) -> Option<Self>;
}

macro_rules! impl_integer {
	($($ty:ty)*) => {
		$(impl const Integer for $ty {
			const MAX: Self = <$ty>::MAX;
			const MIN: Self = <$ty>::MIN;
			const SIZE_BITS: u8 = <$ty>::BITS as _;
			const SIZE_BYTES: u8 = <$ty>::BITS as u8 / 8;
			#[allow(unused_comparisons)]
			const SIGNED: bool = <$ty>::MIN < 0;

			const ZERO: Self = 0;
			const ONE: Self = 1;
			const TWO: Self = 2;
			const THREE: Self = 3;
			const FOUR: Self = 4;
			const FIVE: Self = 5;

			fn saturating_add(self, rhs: Self) -> Self {
				self.saturating_add(rhs)
			}
			fn saturating_sub(self, rhs: Self) -> Self {
				self.saturating_sub(rhs)
			}
			fn saturating_div(self, rhs: Self) -> Self {
				self.saturating_div(rhs)
			}
			fn saturating_mul(self, rhs: Self) -> Self {
				self.saturating_mul(rhs)
			}

			fn checked_add(self, rhs: Self) -> Option<Self> {
				self.checked_add(rhs)
			}
			fn checked_sub(self, rhs: Self) -> Option<Self> {
				self.checked_sub(rhs)
			}
			fn checked_div(self, rhs: Self) -> Option<Self> {
				self.checked_div(rhs)
			}
			fn checked_mul(self, rhs: Self) -> Option<Self> {
				self.checked_mul(rhs)
			}
		})*
	};
}

impl_integer!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize);
