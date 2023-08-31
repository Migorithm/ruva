#![macro_use]
extern crate lib_macros;
extern crate lib_traits;

pub mod prelude {
	pub use lib_macros::Message;

	pub use lib_traits::lib_components::{Aggregate, Command, MailSendable, Message, MessageMetadata, OutBox};
	pub use lib_traits::{convert_event, AggregateMacro, Entity, MailSendableMacro};
}
