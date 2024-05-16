use proc_macro::TokenStream;

use quote::ToTokens;
use syn::{parse::Parser, parse_macro_input, Data, DataStruct, DeriveInput, Field, Ident};

use crate::utils::{
	check_if_field_has_attribute_and_return_field_name, extracts_field_names_from_derive_input, locate_crate_on_derive_macro, remove_fields_from_fields_based_on_field_name, skip_over_attributes,
};

pub(crate) fn render_aggregate(input: TokenStream) -> TokenStream {
	let mut ast = parse_macro_input!(input as DeriveInput);
	let name = ast.ident.clone();
	let crates = locate_crate_on_derive_macro(&ast);

	let adapter_quote = create_struct_adapter_quote(&ast);

	if let syn::Data::Struct(DataStruct {
		fields: syn::Fields::Named(ref mut fields),
		..
	}) = &mut ast.data
	{
		fields.named.iter_mut().for_each(|f| {
			skip_over_attributes(f, "adapter_ignore");
		});

		if fields.named.iter().any(|x| x.ident.as_ref().unwrap() == "is_existing") {
			panic!("is_existing field not injectable! Perhaps it's duplicated?");
		}
		if fields.named.iter().any(|x| x.ident.as_ref().unwrap() == "events") {
			panic!("events field not injectable! Perhaps it's duplicated?");
		}
		if fields.named.iter().any(|x| x.ident.as_ref().unwrap() == "version") {
			panic!("version field not Injectable! Perhaps it's duplicated?");
		}

		fields.named.extend([
			syn::Field::parse_named
				.parse2(quote! {
				   #[serde(skip_deserializing, skip_serializing)]
				   pub(crate) is_existing: bool
				})
				.unwrap(),
			syn::Field::parse_named
				.parse2(quote! {
				   #[serde(skip_deserializing, skip_serializing)]
				   pub(crate) is_updated: bool
				})
				.unwrap(),
			syn::Field::parse_named
				.parse2(quote! {
				   #[serde(skip_deserializing, skip_serializing)]
				   pub(crate) events: ::std::collections::VecDeque<::std::sync::Arc<dyn #crates::prelude::TEvent>>
				})
				.unwrap(),
			syn::Field::parse_named
				.parse2(quote! {
				   pub(crate) version: i32
				})
				.unwrap(),
		]);
	} else {
		panic!("[entity] can be attached only to struct")
	}

	// if identifier_types.is_empty() {
	// 	panic!("identifer must be speicified!")
	// } else if identifier_types.len() > 1 {
	// 	panic!("identifer is specified only once!")
	// }
	// let aggregate_identifier_type = identifier_types.first().unwrap();

	let setters = get_setters(&ast.data);

	quote!(
		#ast
		impl #crates::prelude::TAggregate for #name{
			// type Identifier = #aggregate_identifier_type;

			fn events(&self) -> &::std::collections::VecDeque<::std::sync::Arc<dyn #crates::prelude::TEvent>> {
				&self.events
			}
			fn take_events(&mut self) -> ::std::collections::VecDeque<::std::sync::Arc<dyn #crates::prelude::TEvent>> {
				::std::mem::take(&mut self.events)
			}
			fn raise_event(&mut self, event: ::std::sync::Arc<dyn #crates::prelude::TEvent>) {
				tracing::info!("event raised! {:?}", event.metadata());
				self.events.push_back(event)
			}
		}

		impl #name{
			#setters
		}


		#adapter_quote
	)
	.into()
}

pub(crate) fn render_entity_token(input: TokenStream) -> TokenStream {
	let mut ast = parse_macro_input!(input as DeriveInput);
	let name = &ast.ident;

	if let syn::Data::Struct(DataStruct {
		fields: syn::Fields::Named(ref mut fields),
		..
	}) = &mut ast.data
	{
		fields.named.push(
			syn::Field::parse_named
				.parse2(quote! {

				   #[serde(skip_deserializing, skip_serializing)]
				   pub(crate) is_existing: bool

				})
				.unwrap(),
		);
		fields.named.push(
			syn::Field::parse_named
				.parse2(quote! {

				   #[serde(skip_deserializing, skip_serializing)]
				   pub(crate) is_updated: bool

				})
				.unwrap(),
		);
	} else {
		panic!("[entity] can be attached only to struct")
	}

	let setters = get_setters(&ast.data);
	quote!(
		#ast
		impl #name{
			#setters
		}
	)
	.into()
}

fn get_setters(data: &Data) -> proc_macro2::TokenStream {
	let field_idents: Vec<Field> = match data {
		Data::Struct(data) => data.fields.clone().into_iter().filter_map(Some).collect(),
		_ => panic!("Only Struct Is supported"),
	};
	let mut quotes = vec![];
	for f in field_idents {
		let ident = f.ident.unwrap();
		let ty = f.ty.to_token_stream().to_string();
		let code = format!(
			"pub fn set_{}(&mut self, {}:impl core::convert::Into<{}>){{self.{}={}.into();self.is_updated=true}}",
			ident, ident, ty, ident, ident
		);
		quotes.push(code);
	}
	let joined: proc_macro2::TokenStream = quotes.join(" ").parse().unwrap();
	joined
}

pub fn create_struct_adapter_quote(input: &DeriveInput) -> proc_macro2::TokenStream {
	let aggregate_name = input.ident.clone();
	let adapter_name = Ident::new(&(input.ident.to_string() + "Adapter"), proc_macro2::Span::call_site());

	let mut adapter_input = input.clone();
	adapter_input.ident = adapter_name.clone();

	let mut fields_to_ignore: Vec<String> = vec![];

	if let syn::Data::Struct(DataStruct {
		fields: syn::Fields::Named(ref mut fields),
		..
	}) = &mut adapter_input.data
	{
		fields.named.iter_mut().for_each(|f: &mut Field| {
			if let Some(ignorable_field) = check_if_field_has_attribute_and_return_field_name(f, "adapter_ignore") {
				fields_to_ignore.push(ignorable_field);
				skip_over_attributes(f, "adapter_ignore");
			}
		});
		remove_fields_from_fields_based_on_field_name(fields, &fields_to_ignore);
	}

	let mut aggregates_fields: Vec<String> = vec![];
	let mut adapter_fields: Vec<String> = vec![];

	extracts_field_names_from_derive_input(input).into_iter().for_each(|field_name| {
		// ignorable field means that the field is not compatible with adapter
		if !fields_to_ignore.contains(&field_name) {
			adapter_fields.push(format!("{}: value.{}", field_name, field_name));
			aggregates_fields.push(format!("{}: value.{}", field_name, field_name));
		}
	});

	aggregates_fields.push("is_existing: true".to_string());
	aggregates_fields.push("is_updated: false".to_string());
	aggregates_fields.push("events: ::std::collections::VecDeque::new()".to_string());
	aggregates_fields.push("version: 0".to_string());

	if !fields_to_ignore.is_empty() {
		aggregates_fields.push("..Default::default()".to_string());
	}
	let aggregates_fields = aggregates_fields.join(",");
	let adapter_fields = adapter_fields.join(",");

	let aggregates_fields: proc_macro2::TokenStream = aggregates_fields.parse().unwrap();

	let adapter_fields: proc_macro2::TokenStream = adapter_fields.parse().unwrap();

	quote!(

		#adapter_input

		impl From<#adapter_name> for #aggregate_name{
			fn from(value: #adapter_name) -> Self{
				Self{
					#aggregates_fields
				}
			}
		}

		impl From<#aggregate_name> for #adapter_name{
			fn from(value: #aggregate_name) -> Self{
				Self{
					#adapter_fields
				}
			}
		}
	)
}
