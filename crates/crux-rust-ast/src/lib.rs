#![feature(proc_macro_totokens)]

pub use {proc_macro2::*, quote::quote};

pub mod external {
	//! External crates imported by `crux-rust-ast`.

	pub use {proc_macro2, quote};
	pub extern crate proc_macro;
}
pub mod ast;

use std::{fmt::Debug, iter::Peekable};

pub trait TokenIter: Iterator<Item = TokenTree> + Clone {
	fn next_is_ident(&mut self, ident: &str) -> bool;
	fn next_is_punct(&mut self, punct: char) -> bool;
	fn next_is_group_with_delimiter(&mut self, delimiter: Delimiter) -> bool;
	fn peek(&mut self) -> Option<&TokenTree>;
}
impl<T: Iterator<Item = TokenTree> + Clone> TokenIter for Peekable<T> {
	fn next_is_ident(&mut self, ident: &str) -> bool {
		self.peek().is_some_and(|next| match next {
			TokenTree::Ident(next_ident) => next_ident.to_string().as_str().eq(ident),
			_ => false,
		})
	}
	fn next_is_punct(&mut self, punct: char) -> bool {
		self.peek().is_some_and(|next| match next {
			TokenTree::Punct(next_punct) => next_punct.as_char().eq(&punct),
			_ => false,
		})
	}
	fn next_is_group_with_delimiter(&mut self, delimiter: Delimiter) -> bool {
		self.peek().is_some_and(|next| match next {
			TokenTree::Group(group) => group.delimiter().eq(&delimiter),
			_ => false,
		})
	}
	fn peek(&mut self) -> Option<&TokenTree> {
		Peekable::peek(self)
	}
}

pub trait AstComponent: Sized {
	type ParseError: Copy + Eq + Debug;

	fn is_next(iter: &mut impl TokenIter) -> bool;

	fn maybe_parse(iter: &mut impl TokenIter) -> Option<Result<Self, Self::ParseError>> {
		Self::is_next(iter).then(|| Self::parse(iter))
	}
	fn parse(iter: &mut impl TokenIter) -> Result<Self, Self::ParseError>;

	fn maybe_skip(iter: &mut impl TokenIter) {
		if Self::is_next(iter) {
			Self::skip(iter);
		}
	}
	fn skip(iter: &mut impl TokenIter);
}

#[macro_export]
macro_rules! parse {
	($src:expr => $($t:tt)*) => {{
		parse!(@impl $($t)*);
	}};

	(@impl _ $skip:ty, $($t:tt)*) => {
		parse!(@impl $($t)*);
	};
	(@impl ? $optional:ty, $($t:tt)*) => {
		parse!(@impl $($t)*);
	};
	(@impl $parse:ty, $($t:tt)*) => {
		parse!(@impl $($t)*);
	};
}
