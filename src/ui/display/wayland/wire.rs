//! Implementation of Wayland's wire format, which is used to send messages
//! between the client and compositor.
//!
//! Wayland docs: https://wayland.freedesktop.org/docs/html/ch04.html#sect-Protocol-Wire-Format

use {crate::lang::Infallible, core::iter::Extend};

pub trait ToWire: Sized {
	type Error;

	fn to_wire(&self, buffer: &mut impl Extend<u8>) -> Result<(), Self::Error>;
}
pub trait FromWire<'a>: Sized {
	type Error;

	fn from_wire(buffer: &'a [u8]) -> Result<(u16, Self), Self::Error>;
}

impl FromWire<'_> for u32 {
	type Error = Infallible;

	fn from_wire(buffer: &[u8]) -> Result<(u16, Self), Self::Error> {
		let bytes = &buffer[..4];
		Ok((
			4,
			Self::from_ne_bytes(unsafe { bytes.try_into().unwrap_unchecked() }),
		))
	}
}
impl ToWire for u32 {
	type Error = Infallible;

	fn to_wire(&self, buffer: &mut impl Extend<u8>) -> Result<(), Self::Error> {
		buffer.extend(self.to_ne_bytes());
		Ok(())
	}
}
impl FromWire<'_> for i32 {
	type Error = Infallible;

	fn from_wire(buffer: &[u8]) -> Result<(u16, Self), Self::Error> {
		let bytes = &buffer[..4];
		Ok((
			4,
			Self::from_ne_bytes(unsafe { bytes.try_into().unwrap_unchecked() }),
		))
	}
}
impl ToWire for i32 {
	type Error = Infallible;

	fn to_wire(&self, buffer: &mut impl Extend<u8>) -> Result<(), Self::Error> {
		buffer.extend(self.to_ne_bytes());
		Ok(())
	}
}

impl<'a> FromWire<'a> for &'a str {
	type Error = core::str::Utf8Error;

	fn from_wire(buffer: &'a [u8]) -> Result<(u16, Self), Self::Error> {
		let Ok((_, mut len)) = u32::from_wire(buffer);
		let str = crate::text::str_from_utf8(&buffer[4..len as usize + 4])?;

		// strings must be padded to 4 bytes
		if !len.is_multiple_of(4) {
			len += len % 4;
		}

		Ok((4 + len as u16, str))
	}
}
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum StringToWireError {
	InteriorNullByte,
	TooLarge,
}
impl ToWire for &str {
	type Error = StringToWireError;

	fn to_wire(&self, buffer: &mut impl Extend<u8>) -> Result<(), Self::Error> {
		let mut len: u32 = self.len().try_into().or(Err(StringToWireError::TooLarge))?;

		if self.contains('0') {
			return Err(StringToWireError::InteriorNullByte);
		}

		// strings must be padded to 4 bytes
		if !len.is_multiple_of(4) {
			len += len % 4;
		}

		len.to_wire(buffer);
		buffer.extend(self.as_bytes().iter().copied());

		Ok(())
	}
}
