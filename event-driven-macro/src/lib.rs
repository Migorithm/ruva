use aggregate::render_aggregate_token;
use message::{render_event_visibility, render_message_token};

use proc_macro::TokenStream;
use syn::DeriveInput;
#[macro_use]
extern crate quote;
mod aggregate;
mod message;

#[proc_macro_derive(Message, attributes(internally_notifiable, externally_notifiable))]
pub fn message_derive(attr: TokenStream) -> TokenStream {
	let ast: DeriveInput = syn::parse(attr.clone()).unwrap();
	let propagatability = render_event_visibility(&ast);

	render_message_token(&ast, propagatability)
}

#[proc_macro_derive(Aggregate)]
pub fn aggregate_derive(attr: TokenStream) -> TokenStream {
	let ast: DeriveInput = syn::parse(attr.clone()).unwrap();

	render_aggregate_token(&ast)
}
