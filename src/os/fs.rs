//! Items for interacting with filesystems.

use crate::lang::{Borrow, ToOwned};

#[derive(PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct PathSlice(str);
impl ToOwned for PathSlice {
	type Owned = Path;

	fn to_owned(&self) -> Self::Owned {
		Path(self.0.to_owned())
	}
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Path(String);
impl Borrow<PathSlice> for Path {
	fn borrow(&self) -> &PathSlice {
		self
	}
}
impl Deref for Path {
	type Target = PathSlice;

	fn deref(&self) -> &Self::Target {
		unsafe { crate::lang::transmute(self.0.as_str()) }
	}
}
