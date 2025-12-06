//! Items for working with Rust's syntax tree.

use std::convert::Infallible;

use crate::{AstComponent, Delimiter, Group, Ident, Span, TokenIter, TokenStream, TokenTree};

//
//
// AST Components
//
//

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Safety {
	Safe,
	Unsafe,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mutability {
	Mut,
	Const,
}

/// The `pub` keyword - controls which modules can access an item.
///
/// See: https://doc.rust-lang.org/reference/visibility-and-privacy.html
#[derive(Clone, Debug)]
pub enum Visibility {
	/// The item was declared public with `pub`.
	Public,
	/// The item was declared public to some specific module, e.g. `pub(crate)`,
	/// `pub(super)`, or `pub(in some::module)`.
	Scoped(Group),
	/// The item wasn't declared with `pub`.
	Private,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VisibilityParseError {}
impl AstComponent for Visibility {
	type ParseError = VisibilityParseError;

	fn is_next(_: &mut impl TokenIter) -> bool {
		// Since the `pub` token may or may not be present, there's essentially
		// always a visibility AST next.
		true
	}

	fn maybe_parse(iter: &mut impl TokenIter) -> Option<Result<Self, VisibilityParseError>> {
		Some(Self::parse(iter))
	}

	fn parse(iter: &mut impl TokenIter) -> Result<Self, VisibilityParseError> {
		if iter.next_is_ident("pub") {
			iter.next();
			if iter.next_is_group_with_delimiter(Delimiter::Parenthesis) {
				let Some(TokenTree::Group(group)) = iter.next() else {
					unreachable!()
				};
				Ok(Self::Scoped(group))
			} else {
				Ok(Self::Public)
			}
		} else {
			Ok(Self::Private)
		}
	}

	fn maybe_skip(iter: &mut impl TokenIter) {
		Self::skip(iter)
	}

	fn skip(iter: &mut impl TokenIter) {
		if iter.next_is_ident("pub") {
			iter.next();
			if iter.next_is_group_with_delimiter(Delimiter::Parenthesis) {
				iter.next();
			}
		}
	}
}

#[derive(Debug)]
pub struct Generics {
	pub types: Vec<GenericItem>,
}
impl AstComponent for Generics {
	type ParseError = Infallible;

	fn is_next(iter: &mut impl TokenIter) -> bool {
		iter.next_is_punct('<')
	}

	fn parse(iter: &mut impl TokenIter) -> Result<Self, Self::ParseError> {
		todo!()
	}
	fn skip(iter: &mut impl TokenIter) {
		todo!()
	}
}

// TODO this is incomplete
// https://doc.rust-lang.org/reference/items/generics.html
#[derive(Debug)]
pub enum GenericItem {
	Const {
		name: String,
		r#type: Type,
	},
	Type {
		name: String,
	},
	Lifetime {
		lifetime: Lifetime,
		bounds: Vec<Lifetime>,
	},
}

#[derive(Debug)]
pub enum Lifetime {
	Implicit,
	Static,
	Custom(String),
}

/// TODO.
#[derive(Debug)]
pub struct WhereClause {}

/// An attribute macro on a Rust item, e.g. `#[derive(Debug)]`.
///
/// See: https://doc.rust-lang.org/reference/attributes.html
#[derive(Debug)]
pub struct Attribute {
	/// The name of the macro, e.g. `derive` for `#[derive(Debug)]`.
	pub name: String,
	/// If the attribute was declared unsafe. When true, the attribute was
	/// written surrounded by `unsafe()` e.g.
	/// `#[unsafe(export_name = "something")]`.
	pub is_unsafe: bool,
	/// Any arguments passed to the macro in parentheses, e.g. `(Debug)` for
	/// `#[derive(Debug)]`.
	pub args: Option<AttributeArgs>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AttributeParseError {
	MissingBrackets,
	MissingAttributeName,
	MissingUnsafeInner,
}
impl AstComponent for Attribute {
	type ParseError = AttributeParseError;

	fn is_next(iter: &mut impl TokenIter) -> bool {
		iter.next_is_punct('#')
	}

	fn parse(iter: &mut impl TokenIter) -> Result<Self, Self::ParseError> {
		iter.next();
		let Some(TokenTree::Group(group)) = iter.next() else {
			return Err(AttributeParseError::MissingBrackets);
		};
		if group.delimiter() != Delimiter::Bracket {
			return Err(AttributeParseError::MissingBrackets);
		}

		let mut iter = group.stream().into_iter().peekable();

		let Some(TokenTree::Ident(name)) = iter.next() else {
			return Err(AttributeParseError::MissingAttributeName);
		};

		let name = name.to_string();
		Ok(if name.as_str() == "unsafe" {
			let Some(TokenTree::Group(attribute_inner)) = iter.next() else {
				return Err(AttributeParseError::MissingUnsafeInner);
			};
			if attribute_inner.delimiter() != Delimiter::Parenthesis {
				return Err(AttributeParseError::MissingUnsafeInner);
			}

			let mut iter = attribute_inner.stream().into_iter().peekable();

			let Some(TokenTree::Ident(attribute_name)) = iter.next() else {
				return Err(AttributeParseError::MissingAttributeName);
			};

			Self {
				name: attribute_name.to_string(),
				is_unsafe: true,
				args: AttributeArgs::maybe_parse(&mut iter)
					.map(|res| unsafe { res.unwrap_unchecked() }),
			}
		} else {
			Self {
				name,
				is_unsafe: false,
				args: AttributeArgs::maybe_parse(&mut iter)
					.map(|res| unsafe { res.unwrap_unchecked() }),
			}
		})
	}
	fn skip(iter: &mut impl TokenIter) {
		iter.next(); // Skip #
		iter.next(); // Skip []
	}
}

/// Arguments passed to an attribute.
///
/// Examples:
/// - `"lib"` in `#![crate_type = "lib"]`
/// - `(Debug)` in `#[derive(Debug)]`
#[derive(Debug)]
pub enum AttributeArgs {
	/// The attribute was passed delimited arguments (usually arguments
	/// surrounded by parentheses). For example, `#[derive(Debug)]` would use
	/// this.
	Delimited(TokenTree),
	/// The attribute was passed arguments after an equals sign. For example,
	/// `#![crate_type = "lib"]` would use this.
	// TODO: Technically this should store an expression, not any stream of
	// tokens.
	Assigned(TokenStream),
}
impl AstComponent for AttributeArgs {
	type ParseError = Infallible;

	fn is_next(iter: &mut impl TokenIter) -> bool {
		iter.next_is_punct('=') || iter.next_is_group_with_delimiter(Delimiter::Parenthesis)
	}
	fn parse(iter: &mut impl TokenIter) -> Result<Self, Self::ParseError> {
		if iter.next_is_punct('=') {
			iter.next();
			Ok(Self::Assigned(iter.collect()))
		} else {
			Ok(Self::Delimited(iter.next().unwrap()))
		}
	}
	fn skip(iter: &mut impl TokenIter) {
		if iter.next_is_punct('=') {
			// Iterators are lazy, so we can't just call skip and expect it to
			// actually skip items
			// So we find an item that doesn't exist to force it to skip everything
			iter.find(|_| false);
		} else {
			iter.next();
		}
	}
}

#[derive(Debug)]
pub enum Type {
	/// Examples:
	/// - `fn()`
	/// - `extern "C" fn() -> int`
	/// - `unsafe fn()`
	FunctionPointer {
		higher_ranked_lifetimes: (), // TODO
		safety: Safety,
		abi: Option<String>,
		parameters: Vec<(Option<Ident>, Type)>,
		variadic: bool,
	},
	/// Examples:
	/// - `*const u8`
	/// - `*mut u8`
	/// - `*const c_void`
	Pointer {
		mutability: Mutability,
		inner_type: Box<Type>,
	},
	/// Examples:
	/// - `&str`
	/// - `&mut SomeType`
	Reference {
		mutability: Mutability,
		lifetime: (), // TODO
		inner_type: Box<Type>,
	},
	/// Examples:
	/// - `(i8, u8)`
	/// - `(&str, SomeType)`
	Tuple {
		inner_types: Vec<Type>,
	},
	/// Examples:
	/// - `[u8; 8]`
	/// - `[c_char]`
	Array {
		inner_type: Box<Type>,
		length: Option<usize>,
	},
	Owned {
		name: Ident,
		generics: (), // TODO
	},
	Impl {
		traits: Vec<Ident>,
		use_bound: (), // TODO
	},
	Dyn {
		traits: Vec<Ident>,
	},
	/// `!`
	Never,
}

//
//
// High-level ASTs
//
//

/// A structure declaration.
///
/// Examples:
/// - `struct EmptyStruct;`
/// - `struct TupleStruct(SomeType);`
/// - `struct KeyedStruct { field: SomeType }`
/// - `struct PitaToParse<P: 'static + SomeTrait>{ field: P }`
///
/// See: https://doc.rust-lang.org/reference/items/structs.html
#[derive(Debug)]
pub struct Struct {
	pub kind: StructKind,
}
#[derive(Debug)]
pub enum StructKind {
	Empty,
	Tuple(TupleStruct),
	Keyed(KeyedStruct),
}
/// A struct whose fields are defined in a tuple.
#[derive(Debug)]
pub enum TupleStruct {}
/// A struct whose fields are defined in `key: value` pairs.
#[derive(Debug)]
pub enum KeyedStruct {}

#[derive(Debug)]
pub struct FunctionQualifiers {
	pub is_const: bool,
	pub is_async: bool,
	pub safety: Safety,
	pub abi: Option<String>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FunctionQualifiersParseError {
	MissingAbi,
}
impl AstComponent for FunctionQualifiers {
	type ParseError = FunctionQualifiersParseError;

	fn is_next(iter: &mut impl TokenIter) -> bool {
		let mut iter = iter.clone();
		Self::skip(&mut iter);
		iter.next_is_ident("fn")
	}

	fn parse(iter: &mut impl TokenIter) -> Result<Self, Self::ParseError> {
		let mut this = Self {
			is_const: false,
			is_async: false,
			safety: Safety::Safe,
			abi: None,
		};

		if iter.next_is_ident("const") {
			this.is_const = true;
			iter.next();
		}
		if iter.next_is_ident("async") {
			this.is_async = true;
			iter.next();
		}
		if iter.next_is_ident("safe") {
			iter.next();
		} else if iter.next_is_ident("unsafe") {
			this.safety = Safety::Unsafe;
			iter.next();
		}
		if iter.next_is_ident("extern") {
			iter.next();

			if matches!(iter.peek(), Some(TokenTree::Literal(_))) {
				let Some(TokenTree::Literal(lit)) = iter.next() else {
					unreachable!()
				};
				// TODO: Verify that the literal is a string literal or raw string
				// literal
				// e.g. "extern 1.2" is invalid
				this.abi = Some(lit.to_string());
			} else {
				return Err(FunctionQualifiersParseError::MissingAbi);
			}
		}

		Ok(this)
	}
	fn skip(iter: &mut impl TokenIter) {
		if iter.next_is_ident("const") {
			iter.next();
		}
		if iter.next_is_ident("async") {
			iter.next();
		}
		if iter.next_is_ident("safe") || iter.next_is_ident("unsafe") {
			iter.next();
		}
		if iter.next_is_ident("extern") {
			iter.next();

			if matches!(iter.peek(), Some(TokenTree::Literal(_))) {
				iter.next();
			}
		}
	}
}

/// A function declaration.
///
/// Example:
/// ```rs
/// fn my_function<T: 'static + Copy>(val: T) -> (T, T) {
///     (val, val)
/// }
/// ```
#[derive(Debug)]
pub struct Function {
	pub attributes: Vec<Attribute>,
	pub qualifiers: FunctionQualifiers,
	pub name: String,
	pub generics: Generics,
	pub parameters: TokenTree,
	pub return_type: Type,
	pub where_clause: WhereClause,
	pub body: Option<TokenTree>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FunctionParseError {}
impl AstComponent for Function {
	type ParseError = FunctionParseError;

	fn is_next(iter: &mut impl TokenIter) -> bool {
		FunctionQualifiers::is_next(iter)
	}

	fn parse(iter: &mut impl TokenIter) -> Result<Self, Self::ParseError> {
		todo!()
	}
	fn skip(iter: &mut impl TokenIter) {
		todo!()
	}
}
