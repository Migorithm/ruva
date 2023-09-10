use aggregate::{render_aggregate_token, render_entity_token};

use message::{find_identifier, render_event_visibility, render_message_token};
// use outbox::render_outbox_token;
use proc_macro::TokenStream;
use syn::{DeriveInput, ItemFn};

#[macro_use]
extern crate quote;
mod aggregate;
mod dependency;
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

	error::render_error_token(&ast)
}

#[proc_macro_derive(Entity)]
pub fn entity_derive(attr: TokenStream) -> TokenStream {
	let ast: DeriveInput = syn::parse(attr.clone()).unwrap();

	render_entity_token(&ast)
}

#[proc_macro_derive(Command)]
pub fn command_derive(attr: TokenStream) -> TokenStream {
	let ast: DeriveInput = syn::parse(attr.clone()).unwrap();
	let name = ast.ident;

	quote!(
		impl Command for #name{}
	)
	.into()
}

#[proc_macro_attribute]
pub fn dependency(_: TokenStream, input: TokenStream) -> TokenStream {
	let ast: syn::ItemFn = syn::parse_macro_input!(input as ItemFn);
	dependency::register_dependency(ast)
}
