//! [ruva-core]: https://docs.rs/ruva-core
//! [ruva-macro]: https://docs.rs/ruva-macro
//! [Command]: https://docs.rs/ruva-core/latest/ruva_core/message/trait.Command.html
//! [Event]: https://docs.rs/ruva-core/latest/ruva_core/message/trait.Message.html
//! [MessageBus]: https://docs.rs/ruva-core/latest/ruva_core/messagebus/index.html
//! [Context]: https://docs.rs/ruva-core/latest/ruva_core/messagebus/struct.ContextManager.html
//! [AtomicContextManager]: https://docs.rs/ruva-core/latest/ruva_core/messagebus/type.AtomicContextManager.html
//!
//! A Ruva framework for writing reliable and scalable system.
//!
//! At a high level, it provides a few major components:
//!
//! * Tools for [core components with traits][ruva-core],
//! * [Macros][ruva-macro] for processing events and commands
//!
//!
//! # A Tour of Ruva
//!
//! Ruva consists of a number of modules that provide a range of functionality
//! essential for implementing messagebus-like applications in Rust. In this
//! section, we will take a brief tour, summarizing the major APIs and
//! their uses.
//!
//! ## Command & Event
//! You can register any general struct with [Command] Derive Macro as follows:
//! ```ignore
//! #[derive(Command)]
//! pub struct MakeOrder {
//!     pub user_id: i64,
//!     pub items: Vec<String>,
//! }
//! ```
//! As you attach [Command] derive macro, MessageBus now is going to be able to understand how and where it should
//! dispatch the command to.
//!
//! Likewise, you can do the same thing for Event:
//! ```ignore
//! #[derive(Serialize, Deserialize, Clone, Message)]
//! #[internally_notifiable]
//! pub struct OrderFailed {
//!     #[identifier]
//!     pub user_id: i64,
//! }
//!
//! #[derive(Serialize, Deserialize, Clone, Message)]
//! #[internally_notifiable]
//! pub struct OrderSucceeded{
//!     #[identifier]
//!     pub user_id: i64,
//!     pub items: Vec<String>
//! }
//! ```
//! Note that use of `internally_notifiable`(or `externally_notifiable`) and `identifier` is MUST.
//!
//! * `internally_notifiable` is marker to let the system know that the event should be handled
//! within the application
//! * `externally_notifiable` is to leave `OutBox`.
//! * `identifier` is to record aggregate id.
//!
//!
//!
//! ## Initializing Command Handlers
//! Command handlers are responsible for handling commands in an application, the response of which is sent directly to
//! clients. Commands are imperative in nature, meaning they specify what should be done.
//!
//! ```
//! # const IGNORE_1: &str = stringify! {
//! use ruva::prelude::{init_command_handler, init_event_handler};
//! # };
//! # macro_rules! init_command_handler {
//! #    ($($tt:tt)*) => {}
//! # }
//!
//! init_command_handler!(
//! {
//!    MakeOrder: OrderHandler::make_order,
//!    CancelOrder: OrderHandler::cancel_order
//! }
//! );
//! ```
//! In the example above, you see `MakeOrder` is mapped to `OrderHandler::make_order`, handler in application layer.
//!
//! At this point, imagine you want to handle both success/failure case of the `MakeOrder` command processing.
//! Then you have to think about using event handlers.  
//!
//! ## Registering Event
//!
//! `Event` is a side effect of [Command] or yet another [Event] processing.
//! You can register as many handlers as possible as long as they all consume same type of Event as follows:
//!
//! ### Example
//!
//! ```
//! # macro_rules! init_event_handler {
//! #    ($($tt:tt)*) => {}
//! # }
//! init_event_handler!(
//! {
//!    OrderFaild: [
//!            NotificationHandler::send_mail,
//!            ],
//!    OrderSucceeded: [
//!            DeliveryHandler::checkout_delivery_items,
//!            InventoryHandler::change_inventory_count
//!            ]
//! }
//! );
//! ```
//! In the `MakeOrder` Command Handling, we have either `OrderFailed` or `OrderSucceeded` event with their own processing handlers.
//! Events are raised in the handlers that are thrown to [MessageBus] by [Context].
//! [MessageBus] then loops through the handlers UNLESS `StopSentinel` is received.
//!
//! ## Handler API Example
//!
//! Handlers can be located anywhere as long as they accept two argument:
//!
//! * msg - either [Command] or [Event]
//! * context - [AtomicContextManager]
//!
//! ### Example
//! ```ignore
//! pub async fn make_order(
//!     cmd: MakeOrder,
//!     context: AtomicContextManager,
//! ) -> Result<ServiceResponse, ServiceError> {
//!     let mut uow = UnitOfWork::<Repository<OrderAggregate>, TExecutor>::new(context).await;
//!
//!     let mut order_aggregate = OrderAggregate::new(cmd);
//!     uow.repository().add(&mut task_aggregate).await?;
//!
//!     uow.commit::<ServiceOutBox>().await?;
//!
//!     Ok(().into())
//! }
//! ```
//! But sometimes, you may want to add yet another dependencies. For that, Dependency Injection mechanism has been implemented.
//! So, you can also do something along the lines of:
//! ```ignore
//! pub async fn make_order(
//!     cmd: MakeOrder,
//!     context: AtomicContextManager,
//!     payment_gateway_caller: Box<dyn Fn(String, Value) -> Future<(), ServiceError> + Send + Sync + 'static> //injected dependency
//! ) -> Result<ServiceResponse, ServiceError> {
//!     let mut uow = UnitOfWork::<Repository<OrderAggregate>, TExecutor>::new(context).await;
//!
//!     let mut order_aggregate = OrderAggregate::new(cmd,payment_gateway_caller);
//!     uow.repository().add(&mut task_aggregate).await?;
//!
//!     uow.commit::<ServiceOutBox>().await?;
//!
//!     Ok(().into())
//! }
//! ```
//!
//! How is this possible? because we preprocess handlers so it can allow for `DI container`.
//!
//! ## Dependency Injection
//! You can simply register dependencies by putting attribute on top of free function.
//!
//! ### Example
//!
//! ```ignore
//! // crate::dependencies
//! pub fn payment_gateway_caller() -> Box<dyn Fn(String, Value) -> Future<(), ServiceError> + Send + Sync + 'static> {
//!     if cfg!(test) {
//!         __test_payment_gateway_caller()  //Dependency For Test
//!     } else {
//!         __actual_payment_gateway_caller() //Real Dependency
//!     }
//! }
//! ```
//!
//! This is great as you can take your mind off static nature of the language.
//!
//!
//!
//! ## MessageBus
//! At the core is event driven library is [MessageBus], which gets command and gets raised event from
//! `UnitOfWork` and dispatch the event to the right handlers.
//! As this is done only in framework side, the only way you can 'feel' the presence of messagebus is
//! when you invoke it. Everything else is done magically.
//!
//!
//!
//! ### Example
//! ```ignore
//! #[derive(Command)]
//! pub struct MakeOrder { // Test Command
//!     pub user_id: i64,
//!     pub items: Vec<String>
//! }
//!
//! async fn test_func(){
//!     let bus = MessageBus::new(command_handler(), event_handler())
//!     let command = MakeOrder{user_id:1, items:vec!["shirts","jeans"]}
//!     match bus.handle(command).await{
//!         Err(err)=> { // test for error case }
//!         Ok(val)=> { // test for happy case }
//!     }
//!     }
//!     }
//! }
//! ```
//!
//! #### Error from MessageBus
//! When command has not yet been regitered, it returns an error - `BaseError::NotFound`
//! Be mindful that bus does NOT return the result of event processing as in distributed event processing.

pub extern crate ruva_core;
pub extern crate ruva_macro;
pub extern crate static_assertions;

pub mod prelude {
	pub use ruva_core::event_macros::*;
	pub use ruva_core::message::{Aggregate, Command, Message, MessageMetadata};
	pub use ruva_core::prelude::*;

	pub use ruva_macro::{aggregate, entity, message_handler, Aggregate, ApplicationError, ApplicationResponse, Command, Message};
}

#[cfg(test)]
mod application_error_derive_test {
	use std::fmt::Display;

	use crate as ruva;
	use ruva_core::message::Message;
	use ruva_core::responses::{AnyError, BaseError};
	use ruva_macro::ApplicationError;

	#[derive(Debug, ApplicationError)]
	#[crates(ruva)]
	enum Err {
		#[stop_sentinel]
		Items,
		#[stop_sentinel_with_event]
		StopSentinelWithEvent(Box<dyn Message>),
		#[database_error]
		DatabaseError(Box<AnyError>),
		BaseError(BaseError),
	}

	impl Display for Err {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			match self {
				Self::Items => write!(f, "items"),
				Self::StopSentinelWithEvent(item) => write!(f, "{:?}", item),
				Self::DatabaseError(err) => write!(f, "{:?}", err),
				Self::BaseError(err) => write!(f, "{:?}", err),
			}
		}
	}
}
