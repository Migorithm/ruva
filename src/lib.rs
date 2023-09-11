//! A event-driven framework for writing reliable and scalable system.
//!
//! At a high level, it provides a few major components:
//!
//! * Tools for [core components with traits][event-driven-core],
//! * [Macros][event-driven-macro] for processing events and commands
//!
//! [event-driven-core]: crate::event_driven_core
//! [event-driven-macro]: crate::event_driven_macro
//!
//!
//! # A Tour of Event-Driven-Library
//!
//! Event-Driven-Library consists of a number of modules that provide a range of functionality
//! essential for implementing messagebus-like applications in Rust. In this
//! section, we will take a brief tour, summarizing the major APIs and
//! their uses.
//!
//!
//!
//! ### Registering Command
//!
//! Event-Driven-Library is great for writing applications and handlers that are extensible, and modularic.
//! In Event-Drieven Achitecture, `Command` is a request you are given from enduser whereby user
//! expects to get a return as a result of its processing. Handling `Command` may or may not result in `event`,
//! the result of which is not be directly seen by end user as it is side effect.
//! Therefore, you don't want to string along more than one handler for each `Command`.
//!
//!
//! #### Example
//!
//! ```ignore
//! use event_driven_library::prelude::{init_command_handler,init_event_handler};
//! init_command_handler!(
//! {
//!    Command1: CommandHandler1,
//!    Command2: CommandHandler2,
//!    Command3: CommandHandler3,
//!    Command4: CommandHandler4,
//! }
//! ```
//!
//!
//! ### Registering Event
//!
//! `Event` on the other hand, is a side effect of command processing. You can't predict how many
//! transactions should be processed; for that reason, you have vector of events for each event.
//!
//! #### Example
//!
//! ```ignore
//! init_event_handler!(
//! {
//!    Event1: [
//!            EventHandler1,
//!            EventHandler2,
//!            EventHandler3
//!            ],
//!    Event2: [
//!            EventHandler4 => (mail_sender)
//!            ]
//! }
//! ```
//!
//! ### Dependency Injection
//! As you may have noticed in the example above command handler or event handler may have
//! other dependencies other than message and `Context`(which will be covered later). In this case,
//! You can simply register dependencies by putting attribute on top of free function.
//!
//! #### Example
//!
//! ```ignore
//! #[dependency]
//! pub fn mail_sender() {
//!    ...
//! }
//! ```
//!
//! This is great as you can take your mind off static nature of the language.
//!
//!
//! ### Command & Event
//! You can register any general struct with `Command`[Command] Derive Macro as follows:
//! ```ignore
//! #[derive(Command)]
//! pub struct CustomCommand {
//!     pub id: i64,
//!     pub name: String,
//! }
//! ```
//!
//! Likewise, you can do the same thing for Event:
//! ```ignore
//! #[derive(Serialize, Deserialize, Clone, Message)]
//! #[internally_notifiable]
//! pub struct YourCustomEvent {
//!     #[identifier]
//!     pub user_id: UserId,
//!     pub random_uuid: Uuid,
//! }
//! ```
//! Note that use of `internally_notifiable`(or `externally_notifiable`) and `identifier` is MUST.
//!
//! * `internally_notifiable` is marker to let the system know that the event should be handled
//! within the application
//! * `externally_notifiable` is to leave `OutBox`.
//! * `identifier` is to record aggregate id.
//!
//! [Command]: crate::event_driven_core::message::Command
//! [Message]: crate::event_driven_core::message::Message
//!
//!
//!
//! ### MessageBus
//! `MessageBus`[MessageBus] is central pillar which gets command and gets raised event from
//! `UnitOfWork` and dispatch the event to the right handlers.
//! As this is done only in framework side, the only way you can 'feel' the presence of messagebus is
//! when you invoke it. Everything else is done magically.
//!
//! #### Example
//! ```ignore
//! #[derive(Command)]
//! pub struct TestCommand { // Test Command
//!     pub id: i64,
//!     pub name: String,
//! }
//!
//! async fn test_func(){
//!     let bus = MessageBus::new(command_handler().await, event_handler().await)
//!     let command = TestCommand{id:1,name:"Migo".into()}
//!     let _ = bus.handle(command).await // Use of command
//! }
//! ```
//!
//! #### Error from MessageBus
//! When command has not yet been regitered, it returns an error - `BaseError::CommandNotFound`
//! Be mindful that bus does NOT return the result of event processing as in distributed event processing.
//!
//! [MessageBus]: crate::event_driven_core::messagebus::MessageBus

pub mod prelude {
	pub use event_driven_core::convert_event;
	pub use event_driven_core::event_macros::*;
	pub use event_driven_core::message::{Aggregate, Command, Message, MessageMetadata};
	pub use event_driven_core::prelude::*;
	pub use event_driven_macro::{dependency, Aggregate, ApplicationError, Command, Entity, Message};
}

#[cfg(test)]
mod test_expand {

	#[test]
	fn entity() {
		#[derive(event_driven_macro::Entity, Default)]
		struct SomeEntity {
			age: i64,
			name: String,
		}
		let entity = SomeEntity::default();

		let entity = entity.set_age(10).set_name("MigoLee".to_string());
		assert_eq!(entity.age, 10);
		assert_eq!(entity.name, "MigoLee".to_string())
	}
}
