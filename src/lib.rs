//! [ruva-core]: https://docs.rs/ruva-core
//! [ruva-macro]: https://docs.rs/ruva-macro
//! [TCommand]: https://docs.rs/ruva-core/latest/ruva_core/message/trait.TCommand.html
//! [TEvent]: https://docs.rs/ruva-core/latest/ruva_core/message/trait.TEvent.html
//! [TMessageBus]: https://docs.rs/ruva-core/latest/ruva_core/messagebus/trait.TMessageBus.html
//! [ContextManager]: https://docs.rs/ruva-core/latest/ruva_core/messagebus/struct.ContextManager.html
//! [AtomicContextManager]: https://docs.rs/ruva-core/latest/ruva_core/messagebus/type.AtomicContextManager.html
//! [TCommandService]: https://docs.rs/ruva-core/latest/ruva_core/handler/trait.TCommandService.html
//! [TCommitHook]: https://docs.rs/ruva-core/latest/ruva_core/unit_of_work/trait.TCommitHook.html
//! [into_command]: https://docs.rs/ruva-macro/latest/ruva_macro/attr.into_command.html
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
//! ## Command & Event
//! You can register any general struct with [into_command] attribute macro as follows:
//! ```rust,no_run
//! #[ruva::into_command]
//! pub struct MakeOrder {
//!     pub user_id: i64,
//!     pub items: Vec<String>,
//! }
//! ```
//! Only when you attach [into_command] attribute macro, [TMessageBus] will be able to understand how and where it should
//! dispatch the command to.
//!
//! To specify [TEvent] implementation, annotate struct with `TEvent` derive macro as in the following example:
//! ```rust,ignore
//! use serde::Serialize;
//! use serde::Deserialize;
//! use ruva::serde_json;
//!
//! #[derive(Serialize, Deserialize, Clone, ruva::TEvent)]
//! #[internally_notifiable]
//! pub struct OrderFailed {
//!     pub user_id: i64,
//! }
//!
//! #[derive(Serialize, Deserialize, Clone, ruva::TEvent)]
//! #[externally_notifiable]
//! pub struct OrderSucceeded{
//!     #[identifier]
//!     pub user_id: i64,
//!     pub items: Vec<String>
//! }
//! ```
//! Note that use of `internally_notifiable`(or `externally_notifiable`) and `identifier` are MUST.
//!
//! * `internally_notifiable` is marker to let the system know that the event should be handled
//!    within the application
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
//! ```rust,ignore
//! pub struct MessageBus {
//! event_handler: &'static TEventHandler<ApplicationResponse, ApplicationError>,
//! }
//!
//! impl<C> ruva::TMessageBus<ApplicationResponse,ApplicationError,C> for MessageBus{
//! fn command_handler(
//!     &self,
//!     context_manager: ruva::AtomicContextManager,
//!     cmd: C,
//! ) -> impl ruva::TCommandService<ApplicationResponse, ApplicationError> {
//!         HighestLevelOfAspectThatImplementTCommandService::new(
//!             MidLevelAspectThatImplementTCommandService::new(
//!                 TargetServiceThatImplementTCommandService::new(cmd,other_depdendency)
//!             )
//!         )
//! }
//! }
//! ```
//! For your convenience, Ruva provides declarative macros that handles transaction unit of work as you can use it as follows:
//!
//! ```rust,ignore
//! ruva::register_uow_services!(
//!     MessageBus,
//!     ServiceResponse,
//!     ServiceError,
//!
//!     //Command => handler mapping
//!     CreateUserAccount => create_user_account,
//!     UpdatePassword => update_password,
//!     MakeOrder => make_order,
//!     DeliverProduct => deliver_product
//! )
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
//! ```rust,ignore
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
//! Events are raised in the handlers that are thrown to [TMessageBus] by [ContextManager].
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
//! ```rust,ignore
//! use std::marker::PhantomData;
//! use ruva_core::prelude::TUnitOfWork;
//! use ruva_core::prelude::TRepository;
//!
//!
//! // Service Handler
//! pub struct CustomHandler<R> {
//!     _r: PhantomData<R>,
//! }
//! impl<R> CustomHandler<R>
//! where
//!     R: TRepository + TUnitOfWork,
//! {
//!     pub async fn create_aggregate(
//!         cmd: CreateCommand,
//!         mut uow: R,
//!     ) -> Result<ApplicationResponse, ApplicationError> {
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
//! ```rust,ignore
//! use ruva_core::init_event_handler;
//! use ruva_core::prelude::TEvent;
//! // crate::dependencies
//! init_event_handler!(
//!     ApplicationResponse,
//!     ApplicationError,
//!     |ctx| your_dependency(ctx),
//!
//!     SomethingHappened:[
//!         handle_this_event_handler1,
//!         handle_this_event_handler2,
//!     ],
//!     SomethingElseHappened:[
//!         handle_this_event_handler3,
//!         handle_this_event_handler4,
//!     ],
//! );
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

pub extern crate static_assertions;

pub use ruva_core::__register_uow_services_internal;
pub use ruva_core::init_event_handler;
pub use ruva_core::register_uow_services;

pub use ruva_core::prelude::*;
pub use ruva_core::prepare_bulk_operation;

pub use ruva_macro::{aggregate, entity, event_hook, injectable, into_command, ApplicationError, ApplicationResponse, TConstruct, TEvent};
