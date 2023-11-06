use chrono::{DateTime, Utc};

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct OutBox {
	pub id: Uuid,
	pub aggregate_id: String,
	pub aggregate_name: String,
	pub topic: String,
	pub state: String,
	pub processed: bool,
	pub create_dt: DateTime<Utc>,
}

impl OutBox {
	pub fn new(aggregate_id: String, aggregate_name: String, topic: String, state: String) -> Self {
		Self {
			id: Uuid::new_v4(),
			aggregate_id,
			aggregate_name,
			topic,
			state,
			processed: false,
			create_dt: Default::default(),
		}
	}
}
