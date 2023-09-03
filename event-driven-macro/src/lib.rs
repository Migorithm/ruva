use aggregate::{render_aggregate_token, render_entity_token};
use error::render_error_token;
use message::{find_identifier, render_event_visibility, render_message_token};
use proc_macro::TokenStream;
use syn::DeriveInput;
#[macro_use]
extern crate quote;
mod aggregate;
mod error;
mod message;

#[proc_macro_derive(Message, attributes(internally_notifiable, externally_notifiable, identifier))]
pub fn message_derive(attr: TokenStream) -> TokenStream {
	let ast: DeriveInput = syn::parse(attr.clone()).unwrap();
	let propagatability = render_event_visibility(&ast);
	let identifier = find_identifier(&ast);

	render_message_token(&ast, propagatability, identifier)
}

#[proc_macro_derive(Aggregate)]
pub fn aggregate_derive(attr: TokenStream) -> TokenStream {
	let ast: DeriveInput = syn::parse(attr.clone()).unwrap();

	render_aggregate_token(&ast)
}

#[proc_macro_derive(ApplicationError)]
pub fn error_derive(attr: TokenStream) -> TokenStream {
	let ast: DeriveInput = syn::parse(attr.clone()).unwrap();

	render_error_token(&ast)
}

#[proc_macro_derive(Entity)]
pub fn entity_derive(attr: TokenStream) -> TokenStream {
	let ast: DeriveInput = syn::parse(attr.clone()).unwrap();

	render_entity_token(&ast)
}
