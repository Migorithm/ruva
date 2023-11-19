use message::{find_identifier, get_aggregate_metadata, render_event_visibility, render_message_token};
// use outbox::render_outbox_token;

use proc_macro::TokenStream;

use syn::{DeriveInput, ItemFn};

#[macro_use]
extern crate quote;
mod domain;
mod handler;
mod message;
mod repository;
mod result;
mod utils;

#[proc_macro_derive(TEvent, attributes(internally_notifiable, externally_notifiable, identifier, aggregate))]
pub fn message_derive(attr: TokenStream) -> TokenStream {
	let mut ast: DeriveInput = syn::parse(attr.clone()).unwrap();
	let aggregate_metadata = get_aggregate_metadata(&mut ast);
	let propagatability = render_event_visibility(&ast);
	let identifier = find_identifier(&ast, aggregate_metadata.1);

	render_message_token(&ast, propagatability, identifier, aggregate_metadata.0).into()
}

/// Define TAggregate root
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

/// Attribute macro for marking repository methods that collect events
/// ## Example
/// ```ignore
///
/// #[aggregate]
/// #[derive(Default, Serialize, Deserialize)]
/// struct TestAggregate {
///     #[identifier]
///     pub age: i64,
/// }
///
/// #[async_trait]
/// impl TRepository< TestAggregate> for SqlRepository<TestAggregate> {
///     fn new(executor: Arc<RwLock<SQLExecutor>>) -> Self {
///          ...
///     }
///
///     #[event_hook]
///     async fn update(
///         &mut self,
///         aggregate: &mut TestAggregate,
///     ) -> Result<(), BaseError> {
///         Ok(())
///     }
/// }
///
/// async fn test_event_hook() {
///     '_given: {
///         let mut repo = SqlRepository::new(SQLExecutor::new());
///         let mut aggregate = TestAggregate::default().set_age(64);
///         aggregate.raise_event(SomeEvent { id: aggregate.age }.to_message());
///
///         '_when: {
///             let _ = repo.update(&mut aggregate).await;
///             let events = repo.get_events();
///
///             '_then: {
///                 assert!(!events.is_empty())
///             }
///         }
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn event_hook(_: TokenStream, input: TokenStream) -> TokenStream {
	let ast: ItemFn = syn::parse_macro_input!(input as ItemFn);
	message::event_hook(ast).into()
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

#[proc_macro_derive(TCommand)]
pub fn command_derive(attr: TokenStream) -> TokenStream {
	let ast: DeriveInput = syn::parse(attr.clone()).unwrap();
	let name = ast.ident;

	quote!(
		impl TCommand for #name{}
	)
	.into()
}

#[proc_macro_derive(TRepository)]
pub fn repository_derive(attr: TokenStream) -> TokenStream {
	let ast: DeriveInput = syn::parse(attr.clone()).unwrap();

	repository::render_repository_token(&ast)
}

#[proc_macro_derive(TCommitHook)]
pub fn commit_hook_derive(attr: TokenStream) -> TokenStream {
	let ast: DeriveInput = syn::parse(attr.clone()).unwrap();

	repository::render_event_hook(&ast)
}
