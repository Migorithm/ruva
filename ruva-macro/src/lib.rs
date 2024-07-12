use command::derive_into_command;
use message::{extract_externally_notifiable_event_req, render_event_visibility, render_message_token};
// use outbox::render_outbox_token;

use proc_macro::TokenStream;

use syn::{parse_macro_input, DeriveInput, ItemFn};

#[macro_use]
extern crate quote;
mod command;
mod construct;
mod domain;
mod handler;
mod message;
mod repository;
mod result;
mod utils;

#[proc_macro_derive(TEvent, attributes(internally_notifiable, externally_notifiable, identifier, aggregate))]
pub fn message_derive(attr: TokenStream) -> TokenStream {
	let mut ast: DeriveInput = syn::parse(attr.clone()).unwrap();
	let externally_notifiable_event_req = extract_externally_notifiable_event_req(&mut ast);
	let visibilities = render_event_visibility(&ast);

	render_message_token(&ast, visibilities, externally_notifiable_event_req).into()
}

/// Define TAggregate root
/// ## Example
/// ```rust,no_run
/// #[aggregate]
/// #[derive(Debug, Default, Serialize, Deserialize)]
/// pub struct TestAggregate {
///     pub(crate) age: i64,
/// }
///
/// fn test_aggregate() {
/// let aggregate = TestAggregate::default().set_age(1);
/// assert_eq!(aggregate.version, 0);
/// assert!(!aggregate.is_existing);
/// assert_eq!(aggregate.events.len(), 0);
/// assert_eq!(aggregate.age, 1)
/// }
/// ```
///
/// ```rust,no_run
/// #[aggregate]
/// #[derive(Debug, Default, Serialize, Deserialize)]
/// pub struct TestAggregate {
///     pub(crate) age: i64,
///     pub(crate) name: String,
/// }
/// ```
///
/// Likewise, not specifying `identifier` will also error out
/// ```rust,no_run
/// #[aggregate]
/// #[derive(Debug, Default, Serialize, Deserialize)]
/// pub struct TestAggregate {
///     pub(crate) age: i64,
///     pub(crate) name: String,
/// }
/// ```
///
/// `{your aggregate name}Adapter` will be generated automatically so you can use it to adapt it to database
/// ```rust,no_run```
///
/// #[aggregate]
/// #[derive(Debug, Clone, Serialize, Default)]
/// pub struct AggregateStruct {
///     #[adapter_ignore]
///     id: i32,
///     #[serde(skip_serializing)]
///     name: String,
///     some_other_field: i32,
/// }
/// let aggregate = AggregateStruct::default();
/// let serialized = serde_json::to_string(&aggregate).unwrap();
/// assert_eq!(serialized, "{\"id\":0,\"some_other_field\":0,\"version\":0}");
///
/// let adapter = AggregateStructAdapter::default();
/// let serialized = serde_json::to_string(&adapter).unwrap();
/// assert_eq!(serialized, "{\"some_other_field\":0}");
///
/// ```
///
/// Conversion is automatically done as follows:
/// ```rust,no_run```
/// let aggregate = AggregateStruct {
///         name: "migo".into(),
///         some_other_field: 2,
///         id: 1,
///         ..Default::default()
///     };
///     let converted_adapter = AggregateStructAdapter::from(aggregate);
///     assert_eq!(converted_adapter.name, "migo");
///     assert_eq!(converted_adapter.some_other_field, 2);
///     let converted_struct = AggregateStruct::from(converted_adapter);
///     assert_eq!(converted_struct.name, "migo");
///     assert_eq!(converted_struct.some_other_field, 2);
/// ```
///
/// Generic can also be used for aggregate:
/// ```rust,no_run
/// #[derive(Default, Debug, Serialize, Deserialize)]
/// struct Unset;
///
/// #[aggregate]
/// #[derive(Default, Debug, Serialize, Clone)]
/// struct MyStruct<T = Unset>
/// where
///     T: Send + Sync + Default + 'static,
/// {
///     name: String,
///     age: i32,
///
///     #[adapter_ignore]
///     sub_type: T,
/// }
///
/// impl MyStruct<String> {
///     fn do_something_with_string(&self) -> String {
///         self.sub_type.clone()
///     }
/// }
///
/// impl MyStruct<i32> {
///     fn do_something_with_i32(&self) -> i32 {
///         self.sub_type
///     }
/// }
///
/// let adapter = MyStructAdapter {
///     name: "hello".to_string(),
///     age: 10,
/// };
///
/// let _my_unset_struct = Into::<MyStruct>::into(adapter.clone()); // default type is set which has no method attached.
///
/// let my_string_struct = Into::<MyStruct<String>>::into(adapter.clone());
/// let my_int32_struct = Into::<MyStruct<i32>>::into(adapter.clone());
///
/// assert_eq!(my_string_struct.do_something_with_string(), String::default());
/// assert_eq!(my_int32_struct.do_something_with_i32(), i32::default());
///
/// ```
#[proc_macro_attribute]
pub fn aggregate(_: TokenStream, input: TokenStream) -> TokenStream {
	domain::render_aggregate(input)
}

/// Define ApplicationResponse so that could be recognized by messagebus
/// ## Example
///
/// ```rust,no_run
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
/// ```rust,no_run
///
/// #[aggregate]
/// #[derive(Default, Serialize, Deserialize)]
/// struct TestAggregate {
///     
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
/// ```rust,no_run
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

#[proc_macro_derive(IntoCommand, attributes(required_input))]
pub fn into_command_derive(attr: TokenStream) -> TokenStream {
	let mut ast: DeriveInput = syn::parse(attr.clone()).unwrap();

	let quote = derive_into_command(&mut ast);

	quote!(
		#quote
	)
	.into()
}

#[proc_macro_derive(TRepository)]
pub fn repository_derive(attr: TokenStream) -> TokenStream {
	let ast: DeriveInput = syn::parse(attr.clone()).unwrap();

	repository::render_repository_token(&ast)
}

// what if I want attribute to be #[ruva(except)]?
#[proc_macro_derive(TConstruct, attributes(except))]
pub fn derive_construct(input: TokenStream) -> TokenStream {
	let mut input = parse_macro_input!(input as DeriveInput);

	construct::expand_derive_construct(&mut input).unwrap_or_else(syn::Error::into_compile_error).into()
}
