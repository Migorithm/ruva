use proc_macro2::TokenStream;
use syn::{punctuated::Punctuated, Data, DataStruct, DeriveInput, Fields};

use crate::{
	helpers::generic_helpers::add_sync_trait_bounds,
	utils::{get_attributes, get_type_name},
};

pub(crate) fn derive_into_command(ast: &mut DeriveInput) -> TokenStream {
	let name = &ast.ident;

	let body_name = syn::Ident::new((ast.ident.clone().to_string() + "Body").as_str(), proc_macro2::Span::call_site());

	match &ast.data {
		Data::Struct(DataStruct {
			fields: Fields::Named(syn::FieldsNamed { named, brace_token }),
			struct_token,
			semi_token,
		}) => {
			let input_required_values = named
				.iter()
				.filter(|f| get_attributes(f).into_iter().any(|ident| ident == *"required_input"))
				.cloned()
				.collect::<Punctuated<syn::Field, syn::token::Comma>>();
			let mut body_ast = ast.clone();

			let mut idents_in_vec: Vec<String> = vec![];
			let mut types_in_vec: Vec<String> = vec![];
			let mut input_not_required_ident_type_vec: Vec<String> = vec![];

			body_ast.data = Data::Struct(DataStruct {
				struct_token: *struct_token,
				fields: Fields::Named(syn::FieldsNamed {
					named: named
						.clone()
						.into_iter()
						.map(|f| {
							// Get type name and identifier for the type
							idents_in_vec.push(f.ident.clone().unwrap().to_string());
							types_in_vec.push(get_type_name(&f.ty));
							f
						})
						.filter(|f| {
							!input_required_values
								.iter()
								.any(|required_f| *required_f.ident.clone().unwrap().to_string() == *f.ident.clone().unwrap().to_string())
						})
						.map(|f| {
							input_not_required_ident_type_vec.push(f.ident.clone().unwrap().to_string());
							f
						})
						.collect::<Punctuated<syn::Field, syn::token::Comma>>(),
					brace_token: *brace_token,
				}),
				semi_token: *semi_token,
			});

			body_ast.ident = body_name.clone();

			let mut input_keys_in_vec: Vec<String> = vec![];
			let input_parameters = idents_in_vec
				.iter()
				.zip(types_in_vec.iter())
				.filter(|(key, _value)| !input_not_required_ident_type_vec.contains(key))
				.map(|(key, value)| {
					input_keys_in_vec.push(key.clone());
					format!("{}:{}", key, value)
				})
				.collect::<Vec<_>>()
				.join(",");

			// In case there is no input keys
			let mut input_keys = input_keys_in_vec.join(",");
			if !input_keys.is_empty() {
				input_keys += ",";
			}

			let self_parameters = idents_in_vec
				.iter()
				.zip(types_in_vec.iter())
				.filter(|(key, _value)| input_not_required_ident_type_vec.contains(key))
				.map(|(key, _)| format!("{}:self.{}", key, key))
				.collect::<Vec<_>>()
				.join(",");

			let into_statement: proc_macro2::TokenStream = format!(
				"
                    
			        impl {body_name} {{
			            pub fn into_command(self,{input_parameters}) -> {name}  {{
			                {name}{{
                                {input_keys}
                                {self_parameters}
			                }}
			            }}
			        }}
			        "
			)
			.parse()
			.unwrap();

			quote!(

				#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, utoipa::ToSchema)]
				#body_ast
				#into_statement
			)
		}
		_ => panic!("Only Struct Allowed!"),
	}
}

pub fn declare_command(attr: proc_macro::TokenStream) -> TokenStream {
	let mut ast: DeriveInput = syn::parse(attr.clone()).unwrap();
	let name = ast.ident;

	// add `Send`, `Sync`, `'static` and `std::fmt::Debug` to TypeGenerics if it doesn't have it
	add_sync_trait_bounds(&mut ast.generics);

	let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

	quote!(
		impl #impl_generics #where_clause  TCommand for #name #ty_generics{}
	)
}
