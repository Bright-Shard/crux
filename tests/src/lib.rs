#![feature(prelude_import)]
#![allow(internal_features)]

#[allow(unused_imports)]
#[prelude_import]
use crux::prelude::*;

#[test]
fn log_macro() {
	use crux::{
		lang::borrow::Cow,
		logging::{Log, LogLevel, mklog},
	};
	const MODULE_PATH: &str = crux::lang::module_path!();

	assert_eq!(
		mklog!(LogLevel::Info, "Hello, world!"),
		Log {
			level: LogLevel::Info,
			module: MODULE_PATH,
			msg: Cow::Borrowed("Hello, world!"),
			line: 17,
			column: 3,
			file: "tests/src/lib.rs"
		}
	);
	assert_eq!(
		mklog!(LogLevel::Info, "Hello, {}", "world!"),
		Log {
			level: LogLevel::Info,
			module: MODULE_PATH,
			msg: Cow::Owned(String::from("Hello, world!")),
			line: 28,
			column: 3,
			file: "tests/src/lib.rs"
		}
	);
}

#[test]
#[allow(clippy::assertions_on_constants)]
fn integer_trait() {
	use crux::num::Integer;

	assert!(!u8::SIGNED);
	assert!(i8::SIGNED);
	assert_eq!(u8::MAX, 255);
	assert_eq!(u8::MIN, 0);
	assert_eq!(i8::MAX, 127);
	assert_eq!(i8::MIN, -128);
	assert_eq!(u8::SIZE_BITS, 8);
	assert_eq!(u8::SIZE_BYTES, 1);
}

#[test]
fn arenavec() {
	let vec = ArenaVec::<u32>::new(MemoryAmount::kibibytes(1)).unwrap();

	assert!(vec.is_empty());

	vec.push(69);
	vec.push(420);

	assert!(!vec.is_empty());
	assert_eq!(vec.len(), 2);

	assert_eq!(vec[0], 69);
	assert_eq!(vec[1], 420);
}

#[test]
fn sized_arenavec() {
	let vec = ArenaVec::<u8, u32>::new(MemoryAmount::kibibytes(1)).unwrap();

	assert!(vec.is_empty());
	vec.push(0u8);
	assert_eq!(vec.len(), 1u32);
	vec.push(1u8);
	assert_eq!(vec.len(), 2u32);

	assert_eq!(vec[0u32], 0u8);
	assert_eq!(vec[1u32], 1u8);
	assert_eq!(vec[0u32..=1u32], [0u8, 1u8]);
}

#[test]
fn sized_vec_idx_usize_limit() {
	// When uncommented the below should fail to compile.
	// let vec = SizedVec::<(), u128>::default();
}
