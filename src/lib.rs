#![macro_use]
extern crate event_driven_core;
extern crate event_driven_macro;

pub mod prelude {
	pub use event_driven_core::event_macros::*;
	pub use event_driven_core::lib_components::{Aggregate, Command, MailSendable, Message, MessageMetadata, OutBox};
	pub use event_driven_core::prelude::*;
	pub use event_driven_core::{convert_event, AggregateMacro, Entity, MailSendableMacro};
	pub use event_driven_macro::Message;
}
