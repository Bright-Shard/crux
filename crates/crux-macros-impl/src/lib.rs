use crux_rust_ast::{
	AstComponent, Ident, Span, TokenStream, TokenTree,
	ast::{Attribute, FunctionQualifiers},
	quote,
};

pub fn test(_attr: TokenStream, input: TokenStream) -> TokenStream {
	let mut tokens = input.clone().into_iter().peekable();
	let tokens = &mut tokens;

	while Attribute::is_next(tokens) {
		Attribute::skip(tokens);
	}
	FunctionQualifiers::skip(tokens);
	tokens.next(); // fn keyword

	let Some(TokenTree::Ident(function_name)) = tokens.next() else {
		panic!(); // TODO nicer error
	};

	quote! {
		crux::rt::hook::hook! {
			event: crux::events::run_tests,
			func: #function_name,
			constraints: []
		}
		#input
	}
}
pub fn concat_idents(input: TokenStream) -> TokenStream {
	TokenStream::from_iter([TokenTree::Ident(Ident::new(
		input.to_string().as_str(),
		Span::call_site(),
	))])
}
