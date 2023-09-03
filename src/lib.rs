#![macro_use]
extern crate event_driven_core;
extern crate event_driven_macro;

pub mod prelude {
	pub use event_driven_core::convert_event;
	pub use event_driven_core::event_macros::*;
	pub use event_driven_core::lib_components::{Aggregate, Command, Message, MessageMetadata, OutBox};
	pub use event_driven_core::prelude::*;
	pub use event_driven_macro::{Aggregate, ApplicationError, Entity, Message};
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
