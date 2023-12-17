//! [ruva-core]: https://docs.rs/ruva-core
//! [ruva-macro]: https://docs.rs/ruva-macro
//! [TCommand]: https://docs.rs/ruva-core/latest/ruva_core/message/trait.TCommand.html
//! [TEvent]: https://docs.rs/ruva-core/latest/ruva_core/message/trait.TEvent.html
//! [TMessageBus]: https://docs.rs/ruva-core/latest/ruva_core/messagebus/trait.TMessageBus.html
//! [Context]: https://docs.rs/ruva-core/latest/ruva_core/messagebus/struct.ContextManager.html
//! [AtomicContextManager]: https://docs.rs/ruva-core/latest/ruva_core/messagebus/type.AtomicContextManager.html
//! [TCommandService]: https://docs.rs/ruva-core/latest/ruva_core/handler/trait.TCommandService.html

//!
//! A event-driven framework for writing reliable and scalable system.
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
//! ## TCommand & Event
//! You can register any general struct with [TCommand] Derive Macro as follows:
//! ```ignore
//! #[derive(TCommand)]
//! pub struct MakeOrder {
//!     pub user_id: i64,
//!     pub items: Vec<String>,
//! }
//! ```
//! As you attach [TCommand] derive macro, MessageBus now is going to be able to understand how and where it should
//! dispatch the command to.
//!
//! To specify [TEvent] implementation, annotate struct with `TEvent` derive macro as in the following example:
//! ```ignore
//! #[derive(Serialize, Deserialize, Clone, TEvent)]
//! #[internally_notifiable]
//! pub struct OrderFailed {
//!     #[identifier]
//!     pub user_id: i64,
//! }
//!
//! #[derive(Serialize, Deserialize, Clone, TEvent)]
//! #[internally_notifiable]
//! pub struct OrderSucceeded{
//!     #[identifier]
//!     pub user_id: i64,
//!     pub items: Vec<String>
//! }
//! ```
//! Note that use of `internally_notifiable`(or `externally_notifiable`) and `identifier` are MUST.
//!
//! * `internally_notifiable` is marker to let the system know that the event should be handled
//! within the application
//! * `externally_notifiable` is to leave `OutBox`.
//! * `identifier` is to record aggregate id.
//!
//! This results in the following method attach to the struct for example,
//! * `to_message()` : to convert the struct to heap allocated data structure so messagebus can handle them.
//! * `state()` : to record event's state for outboxing
//!
//!
//! ## Initializing TCommandService
//! For messagebus to recognize service handler, [TCommandService] must be implemented, the response of which is sent directly to
//! clients.
//!
//! ```
//! # const IGNORE_1: &str = stringify! {
//! impl ruva::prelude::TMessageBus<CustomResponse,CustomError,CustomCommand> for MessageBus{
//! fn event_handler(&self) -> &'static ruva::prelude::TEventHandler<CustomResponse, CustomError> {
//!     self.event_handler
//! }
//! fn command_handler(
//!     &self,
//!     context_manager: ruva::prelude::AtomicContextManager,
//! ) -> Box<dyn ruva::prelude::TCommandService<CustomResponse, CustomError, CustomCommand>> {
//!     Box::new(
//!         HighestLevelOfAspectThatImplementTCommandService::new(
//!             MidLevelAspectThatImplementTCommandService::new(
//!                 TargetServiceThatImplementTCommandService
//!             )
//!         )
//!     )
//! }
//! }
//!
//! ```
//!
//! ## Registering Event
//!
//! [TEvent] is a side effect of [TCommand] or yet another [TEvent] processing.
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
//! In the `MakeOrder` TCommand Handling, we have either `OrderFailed` or `OrderSucceeded` event with their own processing handlers.
//! Events are raised in the handlers that are thrown to [TMessageBus] by [Context].
//! [TMessageBus] then loops through the handlers UNLESS `StopSentinel` is received.
//!
//! ## Handler API Example
//!
//! Handlers can be located anywhere as long as they accept two argument:
//! * msg - either [TCommand] or [TEvent]
//! * context - [AtomicContextManager]
//!
//!
//! ### Example
//! ```ignore
//! // Service Handler
//! pub struct CustomHandler<R> {
//!     _r: PhantomData<R>,
//! }
//! impl<R> CustomHandler<R>
//! where
//!     R: TCustomRepository + TUnitOfWork,
//! {
//!     pub async fn create_aggregate(
//!         cmd: CreateCommand,
//!         mut uow: R,
//!     ) -> Result<CustomResponse, CustomError> {
//!         // Transation begin
//!         uow.begin().await?;
//!         let mut aggregate: CustomAggregate = CustomAggregate::new(cmd);
//!         uow.add(&mut aggregate).await?;
//!
//!         // Transation commit
//!         uow.commit().await?;
//!         Ok(aggregate.id.into())
//!     }
//! }
//! ```
//!
//!
//! ## Dependency Injection(For event handlers)
//! For dependency to be injected into handlers, you just need to declare dependencies in `crate::dependencies` and
//! specify identifiers for them. It's worth noting that at the moment, only parameterless function or function that takes
//! [AtomicContextManager] are allowed.
//!
//! ### Example
//!
//! ```ignore
//! // crate::dependencies
//! init_event_handler!({
//!     R: ApplicationResponse,
//!     E: ApplicationError,
//!     {
//!         SomethingHappened:[
//!             // take dependency defined in `crate::dependencies` named `uow` with context being argument
//!             Handler::handle_this_event1 => (uow(c)),
//!             // function that doesn't take additional dependency
//!             Handler::handle_this_event2,
//!         ],
//!         SomethingElseHappened:[
//!             //take dependency defined in `crate::dependencies` named `dependency1`
//!             Handler::handle_this_event3 => (dependency1),
//!             Handler::handle_this_event4 => (dependency1),
//!         ],
//!     }
//! }
//! )
//! ```
//!
//!
//!
//! ## TMessageBus
//! At the core is event driven library is [TMessageBus], which gets command and take raised events from
//! object that implements [TCommitHook] and dispatch the event to the right handlers.
//! As this is done only in framework side, the only way you can 'feel' the presence of messagebus is
//! when you invoke it. Everything else is done magically.
//!
//!
//!
//!
//! #### Error from MessageBus
//! When command has not yet been regitered, it returns an error - `BaseError::NotFound`
//! Be mindful that bus does NOT return the result of event processing as in distributed event processing.

pub extern crate ruva_core;
pub extern crate ruva_macro;
pub extern crate static_assertions;

pub mod prelude {
	pub use ruva_core::event_macros::*;
	pub use ruva_core::message::{EventMetadata, TCommand, TEvent};
	pub use ruva_core::rdb;
	// #[cfg(feature = "sqlx")]
	// pub use ruva_core::rdb;

	pub use ruva_core::prelude::*;

	pub use ruva_macro::{aggregate, entity, event_hook, ApplicationError, ApplicationResponse, TCommand, TEvent, TRepository};
}

#[cfg(test)]
mod test {

	#[test]
	fn application_error_derive_test() {
		use std::fmt::Display;

		use crate as ruva;
		use ruva_core::message::TEvent;
		use ruva_core::responses::{AnyError, BaseError};
		use ruva_macro::ApplicationError;

		#[derive(Debug, ApplicationError)]
		#[crates(ruva)]
		enum Err {
			#[stop_sentinel]
			Items,
			#[stop_sentinel_with_event]
			StopSentinelWithEvent(std::sync::Arc<dyn TEvent>),
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
}
