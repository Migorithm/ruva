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
mod utils;

#[proc_macro_derive(Message, attributes(internally_notifiable, externally_notifiable, identifier))]
pub fn message_derive(attr: TokenStream) -> TokenStream {
	let ast: DeriveInput = syn::parse(attr.clone()).unwrap();
	let propagatability = render_event_visibility(&ast);
	let identifier = find_identifier(&ast);

	render_message_token(&ast, propagatability, identifier).into()
}

#[proc_macro_derive(Aggregate)]
pub fn aggregate_derive(attr: TokenStream) -> TokenStream {
	let ast: DeriveInput = syn::parse(attr.clone()).unwrap();

	render_aggregate_token(&ast)
}

/// Define a Application Error type that can be used in the event-driven-library.
///
/// Before deriving this, you must impl `Debug`traits.
///
/// This macro can be only used in enum.
///
/// ## Attributes
///
/// - `#[crates(...)]` - Specify the name of root of event-driven-library crate. (Default is `event_driven_library`)
/// - `#[stop_sentinel]` - Specify the error matching for `BaseError::StopSentinel`.
/// - `#[stop_sentinel_with_event]` - Specify the error matching for `BaseError::StopSentinelWithEvent`.
/// - `#[database_error]` - Specify the error matching for `BaseError::DatabaseError`.
///
/// ## Example
/// ```ignore
/// #[derive(Debug, ApplicationError)]
/// #[crates(crate::imports::event_driven_library)]
/// enum TestError {
///   #[stop_sentinel]
///   Stop,
///   #[stop_sentinel_with_event]
///   StopWithEvent(Box<AnyError>),
///   #[database_error]
///   DatabaseError(Box<AnyError>),
/// }
/// ```
#[proc_macro_derive(ApplicationError, attributes(stop_sentinel, stop_sentinel_with_event, database_error, crates))]
pub fn error_derive(attr: TokenStream) -> TokenStream {
	let ast: DeriveInput = syn::parse(attr).unwrap();

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
	let ast: ItemFn = syn::parse_macro_input!(input as ItemFn);
	dependency::register_dependency(ast)
}
