#[macro_use]
extern crate macro_rules_attribute;
extern crate macros;

pub mod messagebus;

pub mod repository;
pub mod responses;

pub mod prelude {

	pub use async_trait::async_trait;

	pub use macros::prelude::*;

	pub use paste::paste;
	pub use serde::{Deserialize, Serialize};
}
pub use prelude::*;

#[test]
fn test_aggregate_macro() {
	use macros::prelude::Message;
	use serde::{Deserialize, Serialize};

	#[derive(Debug,Default,Serialize,Deserialize,AggregateMacro!)]
	pub struct SampleAggregate {
		#[serde(skip_deserializing, skip_serializing)]
		events: std::collections::VecDeque<std::boxed::Box<dyn Message>>,
		pub(crate) id: String,
		pub(crate) entity: Vec<Entity>,
	}

	#[derive(Default, Debug, Serialize, Deserialize)]
	pub struct Entity {
		pub(crate) id: i64,
		pub(crate) sub_entity: Vec<SubEntity>,
	}
	#[derive(Default, Debug, Serialize, Deserialize)]
	pub struct SubEntity {
		pub(crate) id: i64,
	}

	let mut aggregate = SampleAggregate::default();
	let mut entity = Entity::default();
	entity.sub_entity.push(SubEntity { id: 1 });
	aggregate.entity.push(entity);

	let res = serde_json::to_string(&aggregate).unwrap();
	println!("{:?}", res)
}
