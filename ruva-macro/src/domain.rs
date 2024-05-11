use proc_macro::TokenStream;

use quote::ToTokens;
use syn::{parse::Parser, parse_macro_input, Data, DataStruct, DeriveInput, Field};

use crate::utils::locate_crate_on_derive_macro;

pub(crate) fn render_aggregate(input: TokenStream) -> TokenStream {
	let mut ast = parse_macro_input!(input as DeriveInput);
	let name = &ast.ident;
	let crates = locate_crate_on_derive_macro(&ast);

	// let mut identifier_types = vec![];
	if let syn::Data::Struct(DataStruct {
		fields: syn::Fields::Named(ref mut fields),
		..
	}) = &mut ast.data
	{
		// fields.named.iter_mut().for_each(|f| {
		// 	identifier_types.extend(find_attr_and_locate_its_type_from_field(f, "identifier"));
		// 	f.attrs.clear();
		// });

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
