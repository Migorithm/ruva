use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_macro_input, FnArg, ImplItem, ItemImpl, ItemTrait, TraitItem};

pub const DELIMETER: &str = "▁DLM▁";
pub const SPECIAL_CHAR: &str = "▁";

pub(crate) fn render_injectable(input: TokenStream, _attrs: TokenStream) -> TokenStream {
	// Parse the input as a trait definition
	let mut input = parse_macro_input!(input as ItemTrait);

	// Collect the signatures of all methods in the trait
	let mut method_signatures = Vec::new();

	let mut generic_map: HashMap<String, Vec<(String, usize)>> = HashMap::new();

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
							if let Some((_generic_idx, generic_ident)) = generics.iter().find(|(_, generic_ident)| type_ident == generic_ident) {
								generic_map.entry(generic_ident.to_string()).or_default().push((method_name.clone(), param_idx));
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

pub(crate) fn render_inject(input: TokenStream, _attrs: TokenStream) -> TokenStream {
	// Parse the input as a trait definition
	let input = parse_macro_input!(input as ItemImpl);

	// Collect the signatures of all methods in the trait

	let trait_info = input.trait_.clone().unwrap().1;

	let mut redefined_methods = Vec::new();

	// Iterate through methods and find where generic parameters are used
	for item in &input.items {
		if let ImplItem::Fn(method) = item {
			let asyncness = match method.sig.asyncness {
				Some(_) => "await",
				None => "",
			};

			let method_name = method.sig.ident.to_string();
			let signature = method.sig.to_token_stream().to_string();
			// extract inputs except for 'self'
			let args: String = method
				.sig
				.inputs
				.iter()
				.skip(1)
				.map(|arg| match arg {
					FnArg::Typed(pat_type) => {
						let pat = pat_type.pat.to_token_stream().to_string();
						if pat.contains("self") {
							"".to_string()
						} else {
							pat
						}
					}
					_ => "".to_string(),
				})
				.collect::<Vec<String>>()
				.join(", ");

			// ! SPECIAL_CHAR is used to indicate the order of the generic parameter
			let formatted_method = format!("{signature} {{ {asyncness} self.{SPECIAL_CHAR}{SPECIAL_CHAR}{SPECIAL_CHAR}.{method_name}({args})}}");

			redefined_methods.push(formatted_method);
		}
	}

	// create trait with `Resolver` postfix and `__TraitName` prefix

	let resolver_name = format!("__{}Resolver", trait_info.to_token_stream());
	// using regex, simplify the logic above
	let re = regex::Regex::new(r"[^a-zA-Z0-9_]+").unwrap();
	let resolver_name = syn::Ident::new(&re.replace_all(&resolver_name, ""), proc_macro2::Span::call_site());

	let redeined_methods = redefined_methods.join("  ");

	let resolver_definition = quote!(
		pub(crate) trait #resolver_name {
		fn resolve(types: &str, order: usize) -> String;
		}



	);

	let resolver_impl = quote!(
		impl<T: #trait_info> #resolver_name for T {
			fn resolve(types: &str, order: usize) -> String {
				let trait_info = stringify!(#trait_info);
				format!(
					"impl<SinglifyType:{trait_info}> {trait_info} for ({types}) {{
						{}
					}}",
					#redeined_methods

				)
			}
		}
	);

	println!("{}", resolver_impl);

	// Convert the expanded output back into a TokenStream
	quote!(
		#input

		#resolver_definition

		#resolver_impl
	)
	.into()
}
