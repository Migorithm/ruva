use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse_macro_input, punctuated::Punctuated, Data, DataStruct, DeriveInput, Fields};

use crate::{
	helpers::{derive_helpers::add_derive_macros_struct_or_enum, generic_helpers::add_sync_trait_bounds},
	utils::{get_attributes, get_type_name, skip_over_attribute_from_derive_input, skip_over_attributes, sort_macros_to_inject, strip_generic_constraints},
};

const COMMAND_CONSTRAINT: [&str; 4] = ["Send", "Sync", "'static", "std::fmt::Debug"];

fn into_command_body(derive_input: &DeriveInput) -> (DeriveInput, proc_macro2::TokenStream) {
	let original_name = &derive_input.ident;

	let mut body_derive = derive_input.clone();

	let ident_generator = |ident: &str| syn::Ident::new(ident, proc_macro2::Span::call_site());

	let body_name = ident_generator(&(derive_input.ident.to_string() + "Body"));

	let Data::Struct(DataStruct {
		fields: Fields::Named(syn::FieldsNamed { named, brace_token }),
		struct_token,
		semi_token,
	}) = &body_derive.data
	else {
		panic!("Only Struct Allowed!");
	};

	let input_required_values = named
		.iter()
		.filter(|f| get_attributes(f).into_iter().any(|ident| ident == *"required_input"))
		.cloned()
		.collect::<Punctuated<syn::Field, syn::token::Comma>>();

	let mut idents_in_vec: Vec<String> = vec![];
	let mut types_in_vec: Vec<String> = vec![];
	let mut input_not_required_ident_type_vec: Vec<String> = vec![];

	body_derive.data = Data::Struct(DataStruct {
		struct_token: *struct_token,
		fields: Fields::Named(syn::FieldsNamed {
			named: named
				.into_iter()
				.cloned()
				.map(|f| {
					// Get type name and identifier for the type
					idents_in_vec.push(f.ident.clone().unwrap().to_string());
					types_in_vec.push(get_type_name(&f.ty));
					f
				})
				.filter(|f| !input_required_values.iter().any(|required_f| required_f.ident == f.ident))
				.map(|mut f| {
					input_not_required_ident_type_vec.push(f.ident.clone().unwrap().to_string());
					skip_over_attributes(&mut f, "required_input");
					f
				})
				.collect::<Punctuated<syn::Field, syn::token::Comma>>(),
			brace_token: *brace_token,
		}),
		semi_token: *semi_token,
	});

	body_derive.ident = body_name.clone();

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

	// Convert the generics to a string
	add_sync_trait_bounds(&mut body_derive.generics, &COMMAND_CONSTRAINT);
	let generics = if body_derive.generics.params.is_empty() {
		String::new()
	} else {
		format!("{}", body_derive.generics.to_token_stream())
	};

	// Convert the where clause to a string (if it exists)
	let where_clause = match &body_derive.generics.where_clause {
		Some(where_clause) => format!("{}", where_clause.to_token_stream()),
		None => String::new(),
	};

	// based on ':', split them and take only the left portion
	// for example, <T:Serialize, U:Deserialize> -> <T, U>
	let generics_with_out_contraints = strip_generic_constraints(&generics);

	let into_statement: proc_macro2::TokenStream = format!(
		"     
		impl {generics} {body_name}{generics_with_out_contraints} {where_clause} {{
			pub fn into_command(self,{input_parameters}) -> {original_name}{generics_with_out_contraints}  {{
				{original_name}{{
					{input_keys}
					{self_parameters}
				}}
			}}
		}}
		"
	)
	.parse()
	.unwrap();

	(body_derive, into_statement)
}

pub fn declare_command(ast: &mut DeriveInput) -> TokenStream {
	let name = ast.ident.clone();

	// add `Send`, `Sync`, `'static` and `std::fmt::Debug` to TypeGenerics if it doesn't have it
	add_sync_trait_bounds(&mut ast.generics, &COMMAND_CONSTRAINT);

	let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

	quote!(
		impl #impl_generics ruva::TCommand for #name #ty_generics #where_clause {}
	)
}

pub fn render_into_command(input: proc_macro::TokenStream, attrs: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let mut macros_to_inject_to_body = vec!["Debug".to_string(), "ruva::Deserialize".to_string()];

	sort_macros_to_inject(&mut macros_to_inject_to_body, attrs);

	let mut ast = parse_macro_input!(input as DeriveInput);

	let (mut body_ast, into_statement) = into_command_body(&ast);
	add_derive_macros_struct_or_enum(&mut body_ast, &macros_to_inject_to_body);

	let macros_to_inject_to_original = ["Debug".to_string(), "ruva::Serialize".to_string()];
	add_derive_macros_struct_or_enum(&mut ast, &macros_to_inject_to_original);
	skip_over_attribute_from_derive_input(&mut ast, "required_input");
	add_sync_trait_bounds(&mut ast.generics, &COMMAND_CONSTRAINT);

	let t_commnad = declare_command(&mut ast);

	quote!(
		#ast

		#t_commnad


		#body_ast


		#into_statement
	)
	.into()
}
