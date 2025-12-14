//! Items dealing with cryptography.

pub use hash::*;

pub mod hash {
	//! Hashing traits and implementations.

	#[allow(deprecated)]
	pub use {
		core::hash::{BuildHasher, BuildHasherDefault, Hash, Hasher, SipHasher},
		hashbrown::DefaultHashBuilder,
	};
	pub type FoldHashBuilder = DefaultHashBuilder;
	pub type FoldHasher = <DefaultHashBuilder as BuildHasher>::Hasher;
}

pub use sha2_const;

// TODO:
// - RNG
// - More hash functions
