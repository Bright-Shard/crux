use core::iter::Step;

use crate::{
	lang::{Add, AddAssign, Copy, Div, DivAssign, Mul, MulAssign, Ord, Sized, Sub, SubAssign},
	text::{Debug, Display},
};

pub trait Type {
	type This;
}
impl<T> Type for T {
	type This = T;
}

//
//
// Function traits
//
//

pub trait FuncOnceGeneric<Args, Ret> {
	fn call_once(self, args: Args) -> Ret;
}
pub trait FuncMutGeneric<Args, Ret>: FuncOnceGeneric<Args, Ret> {
	fn call_mut(&mut self, args: Args) -> Ret;
}
pub trait FuncGeneric<Args, Ret>: FuncMutGeneric<Args, Ret> {
	fn call(&self, args: Args) -> Ret;
}

pub trait FuncOnce: 'static {
	type Args: 'static;
	type Ret: 'static;

	fn call_once(self, args: Self::Args) -> Self::Ret;
}
pub trait FuncMut: FuncOnce {
	fn call_mut(&mut self, args: Self::Args) -> Self::Ret;
}
pub trait Func: FuncMut + Clone {
	fn call(&self, args: Self::Args) -> Self::Ret;
}

macro_rules! impl_funcs {
	() => {};
	($_ignore:ident $($generic:ident)*) => {
		impl<Ty, $($generic,)* Ret> FuncOnceGeneric<($($generic,)*), Ret> for Ty
		where
			Ty: FnOnce($($generic),*) -> Ret
		{
			fn call_once(self, args: ($($generic,)*)) -> Ret {
				#[allow(non_snake_case)]
				let ($($generic,)*) = args;
				self($($generic),*)
			}
		}
		impl<$($generic: 'static,)* Ret: 'static> FuncOnce for fn($($generic),*) -> Ret
		{
			type Args = ($($generic,)*);
			type Ret = Ret;

			fn call_once(self, args: Self::Args) -> Self::Ret {
				#[allow(non_snake_case)]
				let ($($generic,)*) = args;
				self($($generic),*)
			}
		}

		impl<Ty, $($generic,)* Ret> FuncMutGeneric<($($generic,)*), Ret> for Ty
		where
			Ty: FnMut($($generic),*) -> Ret
		{
			fn call_mut(&mut self, args: ($($generic,)*)) -> Ret {
				#[allow(non_snake_case)]
				let ($($generic,)*) = args;
				self($($generic),*)
			}
		}
		impl<$($generic: 'static,)* Ret: 'static> FuncMut for fn($($generic),*) -> Ret
		{
			fn call_mut(&mut self, args: Self::Args) -> Self::Ret {
				#[allow(non_snake_case)]
				let ($($generic,)*) = args;
				self($($generic),*)
			}
		}

		impl<Ty, $($generic,)* Ret> FuncGeneric<($($generic,)*), Ret> for Ty
		where
			Ty: Fn($($generic),*) -> Ret
		{
			fn call(&self, args: ($($generic,)*)) -> Ret {
				#[allow(non_snake_case)]
				let ($($generic,)*) = args;
				self($($generic),*)
			}
		}
		impl<$($generic: 'static,)* Ret: 'static> Func for fn($($generic),*) -> Ret
		{
			fn call(&self, args: Self::Args) -> Self::Ret {
				#[allow(non_snake_case)]
				let ($($generic,)*) = args;
				self($($generic),*)
			}
		}

		impl_funcs!($($generic)*);
	};
}
impl_funcs!(A B C D E F G H I J K L M N O P Q R S T U V W X Y Z);

//
//
// Integer trait
//
//

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
	+ Hash
	+ Step
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

pub const trait UnsignedInteger: Integer {}
pub const trait SignedInteger: Integer {}

impl UnsignedInteger for u8 {}
impl SignedInteger for i8 {}
impl UnsignedInteger for u16 {}
impl SignedInteger for i16 {}
impl UnsignedInteger for u32 {}
impl SignedInteger for i32 {}
impl UnsignedInteger for u64 {}
impl SignedInteger for i64 {}
impl UnsignedInteger for u128 {}
impl SignedInteger for i128 {}
impl UnsignedInteger for usize {}
impl SignedInteger for isize {}
