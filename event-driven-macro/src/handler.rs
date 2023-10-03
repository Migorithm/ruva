use proc_macro::TokenStream;
use syn::{punctuated::Punctuated, token::Comma, FnArg, Ident, ItemFn, Pat, PatIdent, PatType, Path, ReturnType, Signature, Type, TypePath};

pub fn parse_handler(ast: ItemFn) -> TokenStream {
	const OUTPUT_TYPE_NOT_VALID: &str = "#[handler] fn must have valid output type";
	let ItemFn {
		sig: Signature {
			ident,
			output: ReturnType::Type(_, var),
			inputs,
			generics,
			asyncness,
			..
		},
		block,
		..
	} = ast.clone()
	else {
		panic!("{}", OUTPUT_TYPE_NOT_VALID)
	};

	if inputs.is_empty() {
		panic!("There must be message argument!");
	}

	// TODO Check if the first argument is of either Command or Message
	let message = inputs.first().unwrap();
	let args = inputs.iter().skip(1).cloned().collect::<Punctuated<FnArg, Comma>>();
	// extract_dependency(args);

	// Get idents. This will be passed to inner(or redefined) function
	let idents = get_puntuated_idents(inputs.clone());

	// injectables
	let injectables = take_injectables(inputs.clone());

	quote!(
		pub #asyncness fn #ident #generics(#message,context:event_driven_library::prelude::AtomicContextManager)-> #var{
			#asyncness fn inner(#message,#args)->#var{
				#block
			};
			let dependency= crate::dependencies::dependency();
			#(
				let #injectables = dependency.#injectables();
			)*
			inner(#idents).await
		}
	)
	.into()
}

fn get_puntuated_idents(inputs: Punctuated<FnArg, Comma>) -> Punctuated<Ident, Comma> {
	inputs
		.into_iter()
		.filter_map(|i| {
			if let FnArg::Typed(PatType { pat, .. }) = i {
				match *pat {
					Pat::Ident(PatIdent { ref ident, .. }) => Some(syn::Ident::new(&ident.to_string(), proc_macro2::Span::call_site())),
					_ => panic!("Not Allowed!"),
				}
			} else {
				None
			}
		})
		.collect()
}
fn take_injectables(inputs: Punctuated<FnArg, Comma>) -> Vec<proc_macro2::TokenStream> {
	inputs
		.into_iter()
		.skip(1)
		.filter_map(|i| {
			// TODO get attributes on function parameter to sort out context, context-injectable and context-agnostic-injectables
			if let FnArg::Typed(PatType { pat, attrs, .. }) = i {
				match *pat {
					Pat::Ident(PatIdent { ident, .. }) => Some(quote!(#ident)),
					_ => panic!("Not Allowed!"),
				}
			} else {
				None
			}
		})
		.collect::<Vec<_>>()
}
