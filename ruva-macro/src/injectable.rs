use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_macro_input, FnArg, ItemTrait, TraitItem};

pub const DELIMETER: &str = "▁DLM▁";

pub(crate) fn render_injectable(input: TokenStream, _attrs: TokenStream) -> TokenStream {
	// Parse the input as a trait definition
	let mut input = parse_macro_input!(input as ItemTrait);

	// Collect the signatures of all methods in the trait
	let mut method_signatures = Vec::new();

	let mut generic_map: HashMap<usize, Vec<(String, usize)>> = HashMap::new();

	// Collect the generic parameter names (T, U, etc.)
	let generics: Vec<(usize, String)> = input.generics.params.iter().enumerate().map(|(i, param)| (i, param.to_token_stream().to_string())).collect();

	// Iterate through methods and find where generic parameters are used
	for item in &input.items {
		if let TraitItem::Fn(method) = item {
			let method_name = method.sig.ident.to_string();
			method_signatures.push(method.sig.to_token_stream().to_string());

			// Analyze method inputs for generics usage
			for (param_idx, input) in method.sig.inputs.iter().enumerate() {
				if let FnArg::Typed(pat_type) = input {
					if let syn::Type::Path(type_path) = &*pat_type.ty {
						for segment in &type_path.path.segments {
							let type_ident = &segment.ident.to_string();

							// Check if the segment matches any of the generics
							if let Some((generic_idx, _)) = generics.iter().find(|(_, generic_ident)| type_ident == generic_ident) {
								generic_map.entry(*generic_idx).or_default().push((method_name.clone(), param_idx));
							}
						}
					}
				}
			}
		}
	}

	let generic_map = serde_json::to_string(&generic_map).unwrap();

	// Join all method signatures into a single string for __RV_M_SIGNATURE with special character `
	let method_signatures = method_signatures.join(DELIMETER);

	// add `const __RV_M_SIGNATURE: &'static str = #meta_value;` to given trait
	input.items.push(syn::parse_quote! {
		const __RV_M_SIGNATURE: &'static str = #method_signatures;

	});
	input.items.push(syn::parse_quote! {
		const __RV_G_MAP: &'static str = #generic_map;
	});

	// Convert the expanded output back into a TokenStream
	quote!(#input).into()
}
