use crate::prelude::OutBox;
use downcast_rs::{impl_downcast, Downcast};
use std::fmt::Debug;

pub trait TEvent: Sync + Send + Downcast {
	fn externally_notifiable(&self) -> bool {
		false
	}
	fn internally_notifiable(&self) -> bool {
		false
	}

	fn metadata(&self) -> EventMetadata;
	fn outbox(&self) -> OutBox {
		let metadata = self.metadata();
		OutBox::new(metadata.aggregate_id, metadata.aggregate_name, metadata.topic, self.state())
	}

	fn state(&self) -> String;
}

impl_downcast!(TEvent);
impl Debug for dyn TEvent {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.metadata().topic)
	}
}

#[derive(Debug)]
pub struct EventMetadata {
	pub aggregate_id: String,
	pub aggregate_name: String,
	pub topic: String,
}

pub trait TCommand: 'static + Send + Sync + Debug {}
