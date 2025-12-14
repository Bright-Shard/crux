//! A display is a set of interfaces used by GUI applications to appear on
//! screen and get input from the user.

pub mod wayland;

pub trait Display {
	type WindowHandle;

	fn new() -> Self;
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum DisplayType {
	Wayland,
}
