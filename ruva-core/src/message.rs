use crate::prelude::OutBox;
use downcast_rs::{impl_downcast, Downcast};
use serde::Serialize;
use serde_json::Value;
use std::{any::Any, collections::VecDeque, fmt::Debug};

pub trait Message: Sync + Send + Any + Downcast {
	fn externally_notifiable(&self) -> bool {
		false
	}
	fn internally_notifiable(&self) -> bool {
		false
	}

	fn metadata(&self) -> MessageMetadata;
	fn outbox(&self) -> OutBox {
		let metadata = self.metadata();
		OutBox::new(metadata.aggregate_id, metadata.topic, self.state())
	}
	fn message_clone(&self) -> Box<dyn Message>;

	fn state(&self) -> String;

	fn to_message(self) -> Box<dyn Message + 'static>;
}

impl_downcast!(Message);
impl Debug for dyn Message {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.metadata().topic)
	}
}

pub struct MessageMetadata {
	pub aggregate_id: String,
	pub topic: String,
}

// Trait To Mark Event As Mail Sendable. Note that template_name must be specified.
pub trait MailSendable: Message + Serialize + Send + Sync + 'static {
	fn template_name(&self) -> String;
	fn to_json(&self) -> Value {
		serde_json::to_value(self).unwrap()
	}
}

pub trait Command: 'static + Send + Any + Sync + Debug {}

pub trait Aggregate: Send + Sync + Default {
	fn collect_events(&mut self) -> VecDeque<Box<dyn Message>> {
		if !self.events().is_empty() {
			self.take_events()
		} else {
			VecDeque::new()
		}
	}
	fn events(&self) -> &std::collections::VecDeque<Box<dyn Message>>;

	fn take_events(&mut self) -> std::collections::VecDeque<Box<dyn Message>>;
	fn raise_event(&mut self, event: Box<dyn Message>);
}
