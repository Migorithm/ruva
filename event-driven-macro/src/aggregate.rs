use proc_macro::TokenStream;

use quote::ToTokens;
use syn::{Data, DeriveInput, Field};

pub(crate) fn render_aggregate_token(ast: &DeriveInput) -> TokenStream {
	let name = &ast.ident;

	quote!(
		impl Aggregate for #name{

			fn events(&self) -> &std::collections::VecDeque<Box<dyn Message>> {
				&self.events
			}
			fn take_events(&mut self) -> std::collections::VecDeque<Box<dyn Message>> {
				std::mem::take(&mut self.events)
			}
			fn raise_event(&mut self, event: Box<dyn Message>) {
				self.events.push_back(event)
			}

		}
	)
	.into()
}

pub(crate) fn render_entity_token(ast: &DeriveInput) -> TokenStream {
	let name = &ast.ident;

	let field_idents: Vec<Field> = match &ast.data {
		Data::Struct(data) => data.fields.clone().into_iter().filter_map(Some).collect(),
		_ => panic!("Only Struct Is supported"),
	};

	let mut quotes = vec![];

	for f in field_idents {
		let ident = f.ident.unwrap();
		let ty = f.ty.to_token_stream().to_string();

		let code = format!(
			"pub fn set_{}(mut self, {}:{})->Self{{self.{}={}; self }}",
			ident, ident, ty, ident, ident
		);

		quotes.push(code);
	}

	let joined: proc_macro2::TokenStream = quotes.join(" ").parse().unwrap();

	quote!(
		impl #name{
			#joined
		}

	)
	.into()
}
