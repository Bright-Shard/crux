//! Items dealing with cryptography.

pub mod hash {
	//! Hashing traits and implementations.

	#[allow(deprecated)]
	pub use crate::external::core::hash::{
		BuildHasher, BuildHasherDefault, Hash, Hasher, SipHasher,
	};
	pub type FoldHasher = <crate::external::hashbrown::DefaultHashBuilder as BuildHasher>::Hasher;
}

// TODO:
// - RNG
// - More hash functions
