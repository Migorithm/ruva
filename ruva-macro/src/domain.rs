use proc_macro::TokenStream;

use quote::ToTokens;
use syn::{parse::Parser, parse_macro_input, Data, DataStruct, DeriveInput, Field};

use crate::utils::locate_crate_on_derive_macro;

pub(crate) fn render_aggregate_token(ast: &DeriveInput) -> TokenStream {
	let name = &ast.ident;
	let crates = locate_crate_on_derive_macro(ast);

	quote!(
		impl #crates::prelude::Aggregate for #name{

			fn events(&self) -> &::std::collections::VecDeque<Box<dyn #crates::prelude::Message>> {
				&self.events
			}
			fn take_events(&mut self) -> ::std::collections::VecDeque<Box<dyn #crates::prelude::Message>> {
				::std::mem::take(&mut self.events)
			}
			fn raise_event(&mut self, event: Box<dyn Message>) {
				self.events.push_back(event)
			}

		}
	)
	.into()
}

pub(crate) fn render_aggregate(input: TokenStream) -> TokenStream {
	let mut ast = parse_macro_input!(input as DeriveInput);
	let name = &ast.ident;
	let crates = locate_crate_on_derive_macro(&ast);

	if let syn::Data::Struct(DataStruct {
		fields: syn::Fields::Named(ref mut fields),
		..
	}) = &mut ast.data
	{
		fields.named.extend([
			syn::Field::parse_named
				.parse2(quote! {
				   #[serde(skip_deserializing, skip_serializing)]
				   pub(crate) is_existing: bool
				})
				.expect("is_existing field not injectable! Perhaps it's duplicated?"),
			syn::Field::parse_named
				.parse2(quote! {
				   #[serde(skip_deserializing, skip_serializing)]
				   pub(crate) events: ::std::collections::VecDeque<::std::boxed::Box<dyn #crates::prelude::Message>>

				})
				.expect("events field not injectable! Perhaps it's duplicated?"),
			syn::Field::parse_named
				.parse2(quote! {
				   pub(crate) version: i32
				})
				.expect("version field not Injectable! Perhaps it's duplicated?"),
		]);
	} else {
		panic!("[entity] can be attached only to struct")
	}
	let setters = get_setters(&ast.data);

	quote!(
		#ast
		impl #crates::prelude::Aggregate for #name{
			fn events(&self) -> &::std::collections::VecDeque<Box<dyn #crates::prelude::Message>> {
				&self.events
			}
			fn take_events(&mut self) -> ::std::collections::VecDeque<Box<dyn #crates::prelude::Message>> {
				::std::mem::take(&mut self.events)
			}
			fn raise_event(&mut self, event: Box<dyn #crates::prelude::Message>) {
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
			"pub fn set_{}(mut self, {}:impl core::convert::Into<{}>)->Self{{self.{}={}.into(); self }}",
			ident, ident, ty, ident, ident
		);
		quotes.push(code);
	}
	let joined: proc_macro2::TokenStream = quotes.join(" ").parse().unwrap();
	joined
}
