use proc_macro::TokenStream;

macro_rules! def {
	($($(#[doc = $docs:literal])* $kind:ident$(($type:ident))? $macro:ident),* $(,)?) => {
		$(
			def!(@$kind$(($type))? $(#[doc = $docs])* $macro);
		)*
	};

	// Attribute macros
	(@attr $(#[doc = $docs:literal])* $macro:ident) => {
		$(#[doc = $docs])*
		#[proc_macro_attribute]
		pub fn $macro(attr: TokenStream, input: TokenStream) -> TokenStream {
			crux_macros_impl::$macro(attr.into(), input.into()).into()
		}
	};
	// Procedural macros
	(@macro $(#[doc = $docs:literal])* $macro:ident) => {
		$(#[doc = $docs])*
		#[proc_macro]
		pub fn $macro(input: TokenStream) -> TokenStream {
			crux_macros_impl::$macro(input.into()).into()
		}
	};
	// Derive macros
	(@derive($type:ident) $(#[doc = $docs:literal])* $macro:ident) => {
		$(#[doc = $docs])*
		#[proc_macro_derive($type)]
		pub fn $macro(input: TokenStream) -> TokenStream {
			crux_macros_impl::$macro(input.into()).into()
		}
	}
}

def! {
	attr test,
	/// Concatenates the given idents into a single identifier.
	///
	/// ```rs
	/// concat_idents!(s t d); // produces `std`
	/// concat_idents!(s t d)::alloc::String::new();
	/// ```
	macro concat_idents,
}
