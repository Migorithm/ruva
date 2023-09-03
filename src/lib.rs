#![macro_use]
extern crate event_driven_core;
extern crate event_driven_macro;

pub mod prelude {
	pub use event_driven_core::event_macros::*;
	pub use event_driven_core::lib_components::{Aggregate, Command, Message, MessageMetadata, OutBox};
	pub use event_driven_core::prelude::*;
	pub use event_driven_core::{convert_event, Entity};
	pub use event_driven_macro::{Aggregate, ApplicationError, Message};
}
