#![allow(internal_features)] // needed for prelude_import rn
#![allow(clippy::result_unit_err)]
#![feature(allocator_api)]
#![feature(const_trait_impl)]
#![feature(const_ops)]
#![feature(prelude_import)]
#![feature(slice_index_methods)]
#![feature(extend_one)]
#![feature(macro_metavar_expr)]
#![feature(trim_prefix_suffix)]
#![feature(rustc_private)]
#![feature(const_convert)]
#![feature(const_default)]
#![feature(step_trait)]
#![no_main]
#![no_std]

#[cfg(feature = "concurrency")]
pub mod concurrency;
pub mod crypto;
pub mod data_structures;
pub mod ffi;
pub mod lang;
pub mod logging;
pub mod os;
pub mod rt;
#[cfg(feature = "term")]
pub mod term;
pub mod test;

pub extern crate alloc;
pub extern crate core;
pub use crux_macros as macros;

pub mod hooks {
	pub use crate::rt::{entrypoint::call_main, startup_hook};
}
pub mod events {
	pub use crate::rt::{startup, test_harness::run_tests};
}

pub mod prelude {
	//! Exports most useful types/functions in Crux that are unlikely to collide
	//! with the names of existing types/functions.

	pub use crate::{
		self as crux, bitset,
		crypto::hash::Hash,
		data_structures::{
			ArenaString, ArenaVec, BTreeMap, BTreeSet, BinaryHeap, Box, HashMap, HashSet,
			HashTable, SizedVec, TypedVec, Vec, typed_vec_idx,
		},
		lang::{
			AllocError, Allocator, AsyncFn, AsyncFnMut, AsyncFnOnce, Clone, Copy, Default, Deref,
			DerefMut, Drop, Eq, Err, Fn, FnMut, FnOnce, From, Into, IntoIterator, Iterator,
			ManuallyDrop, MaybeUninit, NonNull, NonNullConst, None, Ok, Option, Ord, PartialEq,
			PartialOrd, Result, Send, Sized, Some, Sync, derive, drop, matches, panic, todo,
			transmute, transmute_copy, unreachable,
		},
		logging::{error, fatal, info, trace, warn},
		macros::test,
		os::{
			mem::{GlobalAllocator, MemoryAmount, VirtualMemoryArena},
			proc::{print, println},
		},
		test::{assert, assert_eq, assert_ne, safety_assert, safety_assert_eq, safety_assert_ne},
		text::{CString, Debug, String, format},
	};
}

#[allow(unused_imports)]
#[prelude_import]
use prelude::*;

pub mod io {
	//! General-purpose utilities for transferring data.

	use crate::text::FormatArgs;

	/// Represents a data source that bytes can be transferred into.
	pub trait Writer: Sized {
		/// An error that occurred while using this writer.
		type Error: Debug + PartialEq + Eq;

		/// Transfer bytes into this writer. Bytes will be copied into the
		/// writer's data source. Returns how many bytes were written, or an
		/// error, if one occurred.
		fn write(&mut self, bytes: &[u8]) -> Result<usize, Self::Error>;
		/// Calls [`Writer::write`] continuously until all of the give `bytes`
		/// have been transferred to this writer.
		fn write_all(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
			let mut written = 0;
			let goal = bytes.len();

			loop {
				if written == goal {
					break Ok(());
				}
				written += self.write(&bytes[written..])?;
			}
		}
		/// Write formatted text (e.g. text generated with [`format_args`]) to
		/// this writer.
		///
		/// Note that formatting is implemented in the Rust compiler, and we do
		/// not get to control its error handling, so this method can't return a
		/// nice error type like the other write methods.
		///
		/// [`format_args`]: crate::text::format_args
		fn write_fmt(&mut self, args: FormatArgs) -> Result<(), ()> {
			core::fmt::write(&mut FmtWriter(self), args).map_err(|_| ())
		}
		/// Some data sources need to be "flushed" for written bytes to actually
		/// be transferred. This method would flush the data source so all
		/// written bytes do in fact get transferred.
		fn flush(&mut self) -> Result<(), Self::Error>;
	}

	pub struct FmtWriter<'a>(&'a mut dyn AnyWriter);
	impl core::fmt::Write for FmtWriter<'_> {
		fn write_str(&mut self, s: &str) -> core::fmt::Result {
			self.0.write_all(s.as_bytes()).map_err(|_| core::fmt::Error)
		}
	}

	/// A type-erased version of [`Writer`]. This trait is automatically
	/// implemented for all types that implement [`Writer`].
	pub trait AnyWriter {
		/// Transfer bytes into this writer. Bytes will be copied into the
		/// writer's data source.
		///
		/// Unlike [`Writer::write`], this trait is typed-erase and therefore
		/// does not store a specific error type, so errors are opaque.
		fn write(&mut self, bytes: &[u8]) -> Result<usize, ()>;
		/// Calls [`AnyWriter::write`] continuously until all of the give
		/// `bytes` have been transferred to this writer.
		///
		/// Unlike [`Writer::write_all`], this trait is typed-erase and
		/// therefore does not store a specific error type, so errors are
		/// opaque.
		fn write_all(&mut self, bytes: &[u8]) -> Result<(), ()>;
		/// Write formatted text (e.g. text generated with [`format_args`]) to
		/// this writer.
		///
		/// [`format_args`]: crate::text::format_args
		fn write_fmt(&mut self, args: FormatArgs) -> Result<(), ()>;
		/// Some data sources need to be "flushed" for written bytes to actually
		/// be transferred. This method would flush the data source so all
		/// written bytes do in fact get transferred.
		///
		/// Unlike [`Writer::flush`], this trait is typed-erase and therefore
		/// does not store a specific error type, so errors are opaque.
		fn flush(&mut self) -> Result<(), ()>;
	}
	impl<W> AnyWriter for W
	where
		W: Writer,
	{
		fn write(&mut self, bytes: &[u8]) -> Result<usize, ()> {
			<Self as Writer>::write(self, bytes).map_err(|_| ())
		}
		fn write_all(&mut self, bytes: &[u8]) -> Result<(), ()> {
			<Self as Writer>::write_all(self, bytes).map_err(|_| ())
		}
		fn write_fmt(&mut self, args: FormatArgs) -> Result<(), ()> {
			<Self as Writer>::write_fmt(self, args)
		}
		fn flush(&mut self) -> Result<(), ()> {
			<Self as Writer>::flush(self).map_err(|_| ())
		}
	}
}

pub mod text {
	//! Functions and types for working with text.

	#[doc(inline)]
	pub use {
		alloc::{ffi::CString, fmt::format, format, string::String},
		core::{
			concat,
			ffi::CStr,
			fmt::{
				Arguments as FormatArgs, Debug, Display, Write as TextWrite, write as write_fmt,
			},
			format_args, stringify,
		},
	};

	use crate::lang::{AsStatic, Cow};

	/// Converts the given [`FormatArgs`] to an `&str`, if possible; otherwise
	/// allocates them to a string.
	pub fn maybe_format<'a>(args: FormatArgs<'a>) -> Cow<'a, str> {
		match args.as_str() {
			Some(str) => Cow::Borrowed(str),
			None => Cow::Owned(format(args)),
		}
	}
	pub fn maybe_format_static(args: FormatArgs<'_>) -> Cow<'static, str> {
		match args.as_str() {
			Some(str) => AsStatic::as_static(str),
			None => Cow::Owned(format(args)),
		}
	}
}

#[macro_export]
macro_rules! bitset {
	($($(#[$($struct_attr:tt)*])* $(pub bitset $pub_name:ident)? $(bitset $name:ident)?: $size:ty {$($(#[$($variant_attr:tt)*])* $variant:ident = $val:expr $(,)?)*})*) => {
        $(
        		$(#[$($struct_attr)*])*
        		#[derive(Clone, Copy, PartialEq, Eq)]
          	#[repr(transparent)]
        		$(pub struct $pub_name)? $(struct $name)?($size);
          	impl $($pub_name)? $($name)? {
           		$(
             		$(#[$($variant_attr)*])*
               	pub const $variant: Self = Self($val);
             	)*

            	pub fn contains(self, flag: Self) -> bool {
             		(self.0 & flag.0) == flag.0
               }
	           	pub fn add_flag(self, flag: Self) -> Self {
	             	Self(self.0 | flag.0)
	            }
           	}
            impl $crate::lang::op::BitOr for $($pub_name)? $($name)? {
            	type Output = Self;

             	fn bitor(self, other: Self) -> Self {
              		Self(self.0 | other.0)
               }
            }
        )*
    };
}
