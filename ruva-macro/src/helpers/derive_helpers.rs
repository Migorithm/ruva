use syn::Attribute;

pub fn add_derive_macros_struct_or_enum(input: &mut syn::DeriveInput, macros_to_add: &[String]) {
	let mut derive_paths = vec![];
	// extract meta from input
	let meta_vec = input.attrs.iter().map(|attr| &attr.meta).collect::<Vec<_>>();

	let mut meta_tokens = vec![];
	if let Some(meta) = meta_vec.first() {
		meta_tokens = extract_meta_tokens(meta);
	}

	for macro_to_add in macros_to_add {
		// if it doesn't have in existing derive paths for a given input, push it
		if !meta_tokens.contains(&macro_to_add.split("::").last().unwrap().to_string()) {
			let path = syn::parse_str::<syn::Path>(macro_to_add).unwrap();
			derive_paths.push(path);
		}
	}

	let derive_attr: Attribute = syn::parse_quote!(#[derive(#(#derive_paths),*)]);

	input.attrs.push(derive_attr);
}

fn extract_meta_tokens(meta: &syn::Meta) -> Vec<String> {
	match meta {
		syn::Meta::List(meta_list) => meta_list.tokens.clone().into_iter().map(|token| token.to_string()).collect(),
		_ => vec![],
	}
}
