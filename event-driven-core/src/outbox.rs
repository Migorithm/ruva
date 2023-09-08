use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct OutBox {
	pub id: Uuid,
	pub aggregate_id: String,
	pub topic: String,
	pub state: String,
	pub processed: bool,
	pub create_dt: DateTime<Utc>,
}

impl OutBox {
	pub fn new(aggregate_id: String, topic: String, state: String) -> Self {
		Self {
			id: Uuid::new_v4(),
			aggregate_id,
			topic,
			state,
			processed: false,
			create_dt: Default::default(),
		}
	}
	// fn convert_event(&self) -> Box<dyn Message> {
	// 	// convert event. it takes outbox reference and target type that is to be deserialized.
	// 	// you can insert any number of desired type as long as it is outboxable type.
	// 	convert_event!(self,)
	// }
	pub fn tag_processed(&mut self) {
		self.processed = true
	}

	pub fn id(&self) -> Uuid {
		self.id
	}
	pub fn aggregate_id(&self) -> &str {
		&self.aggregate_id
	}
	pub fn topic(&self) -> &str {
		&self.topic
	}
	pub fn state(&self) -> &str {
		&self.state
	}
	pub fn processed(&self) -> bool {
		self.processed
	}
	pub fn create_dt(&self) -> DateTime<Utc> {
		self.create_dt
	}
}
