//! Testing utilities.

//
//
// Re-exports
//
//

pub use {
	core::{
		assert, assert_eq, assert_ne, debug_assert, debug_assert_eq, debug_assert_ne,
		prelude::rust_2024::test,
	},
	safety_check::*,
};

//
//
// Safety check assertions
//
//

pub mod safety_check {
	//! Perform checks only when the `safety-checks` crate feature is enabled.
	//!
	//! Note that these macros are based on if the `safety-checks` feature is
	//! enabled for *crux*, not the crate where they are invoked.

	#[cfg(safety_checks)]
	#[macro_export]
	macro_rules! safety_assert {
		($ex:expr) => {
			assert!($ex);
		};
	}
	#[cfg(not(safety_checks))]
	#[macro_export]
	macro_rules! safety_assert {
		($ex:expr) => {};
	}
	pub use crate::safety_assert;

	#[cfg(safety_checks)]
	#[macro_export]
	macro_rules! safety_assert_eq {
		($left:expr, $right:expr) => {
			assert_eq!($left, $right);
		};
	}
	#[cfg(not(safety_checks))]
	#[macro_export]
	macro_rules! safety_assert_eq {
		($left:expr, $right:expr) => {};
	}
	pub use crate::safety_assert_eq;

	#[cfg(safety_checks)]
	#[macro_export]
	macro_rules! safety_assert_ne {
		($left:expr, $right:expr) => {
			assert_ne!($left, $right);
		};
	}
	#[cfg(not(safety_checks))]
	#[macro_export]
	macro_rules! safety_assert_ne {
		($left:expr, $right:expr) => {};
	}
	pub use crate::safety_assert_ne;
}
