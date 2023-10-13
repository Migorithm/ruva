use domain::render_aggregate_token;

use message::{find_identifier, render_event_visibility, render_message_token};
// use outbox::render_outbox_token;

use proc_macro::TokenStream;
use syn::{DeriveInput, ItemFn};

#[macro_use]
extern crate quote;
mod domain;

mod handler;
mod message;
mod result;
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

/// Define Aggregate root
/// ## Example
/// ```ignore
/// #[aggregate]
/// #[derive(Debug, Default, Serialize, Deserialize)]
/// pub struct TestAggregate {
///    #[identifier]
///     pub(crate) age: i64,
/// }
///
/// fn test_aggregate() {
/// let aggregate = TestAggregate::default().set_age(1);
/// assert_eq!(aggregate.version, 0);
/// assert!(!aggregate.is_existing);
/// assert_eq!(aggregate.events.len(), 0);
/// assert_eq!(aggregate.age, 1)
/// ```
///
/// the following will cause an error with saying "identifier is specified only once!"
/// ```ignore
/// #[aggregate]
/// #[derive(Debug, Default, Serialize, Deserialize)]
/// pub struct TestAggregate {
///     #[identifier]
///     pub(crate) age: i64,
///      #[identifier]
///     pub(crate) name: String,
/// }
/// ```
///
/// Likewise, not specifying `identifier` will also error out
/// ```ignore
/// #[aggregate]
/// #[derive(Debug, Default, Serialize, Deserialize)]
/// pub struct TestAggregate {
///     pub(crate) age: i64,
///     pub(crate) name: String,
/// }
/// ```
#[proc_macro_attribute]
pub fn aggregate(_: TokenStream, input: TokenStream) -> TokenStream {
	domain::render_aggregate(input)
}

/// Define ApplicationResponse so that could be recognized by messagebus
/// ## Example
/// ```ignore
/// #[derive(Debug, ApplicationResponse)]
/// enum ServiceResponse{
///     Response1
///     Response2
/// }
/// ```
#[proc_macro_derive(ApplicationResponse)]
pub fn response_derive(attr: TokenStream) -> TokenStream {
	let ast: DeriveInput = syn::parse(attr.clone()).unwrap();
	result::render_response_token(&ast)
}

/// Define a Application Error type that can be used in the ruva.
///
/// Before deriving this, you must impl `Debug`traits.
///
/// This macro can be only used in enum.
///
/// ## Attributes
///
/// - `#[crates(...)]` - Specify the name of root of ruva crate. (Default is `ruva`)
/// - `#[stop_sentinel]` - Specify the error matching for `BaseError::StopSentinel`.
/// - `#[stop_sentinel_with_event]` - Specify the error matching for `BaseError::StopSentinelWithEvent`.
/// - `#[database_error]` - Specify the error matching for `BaseError::DatabaseError`.
///
/// ## Example
/// ```ignore
/// #[derive(Debug, ApplicationError)]
/// #[crates(crate::imports::ruva)]
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

	result::render_error_token(&ast)
}

#[proc_macro_attribute]
pub fn entity(_: TokenStream, input: TokenStream) -> TokenStream {
	domain::render_entity_token(input)
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
pub fn message_handler(_: TokenStream, input: TokenStream) -> TokenStream {
	let ast: ItemFn = syn::parse_macro_input!(input as ItemFn);

	handler::parse_handler(ast)
}
