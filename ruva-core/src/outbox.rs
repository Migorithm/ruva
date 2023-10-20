use crate::{prelude::BaseError, unit_of_work::Executor};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;
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

#[async_trait]
pub trait IOutBox<E: Executor>: Send + Sync {
	async fn add(executor: Arc<RwLock<E>>, outboxes: Vec<OutBox>) -> Result<(), BaseError>;
	async fn get() -> Result<Vec<Box<Self>>, BaseError>;
	async fn update(&self, executor: Arc<RwLock<E>>) -> Result<(), BaseError>;
}
