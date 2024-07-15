use std::collections::VecDeque;

use crate::{
	bus_components::contexts::AtomicContextManager,
	prelude::{BaseError, TAggregate, TEvent, TUnitOfWork},
	prepare_bulk_operation,
	repository::TRepository,
};

use super::executor::SQLExecutor;

pub struct SqlRepository {
	pub executor: SQLExecutor,
	events: VecDeque<std::sync::Arc<dyn TEvent>>,
	context: AtomicContextManager,
}

impl SqlRepository {
	pub fn new(context: AtomicContextManager, executor: SQLExecutor) -> Self {
		Self {
			executor,
			events: Default::default(),
			context,
		}
	}
	pub fn event_hook<A: TAggregate>(&mut self, aggregate: &mut A) {
		self.set_events(aggregate.take_events());
	}
	pub(crate) async fn send_internally_notifiable_messages(&self) {
		let event_queue = &mut self.context.write().await;

		self.events.iter().filter(|e| e.internally_notifiable()).for_each(|e| event_queue.push_back(e.clone()));
	}

	pub(crate) async fn save_outbox(&mut self) -> Result<(), BaseError> {
		let outboxes = self.events.iter().filter(|e| e.externally_notifiable()).map(|o| o.outbox()).collect::<Vec<_>>();

		prepare_bulk_operation!(
			&outboxes,
			id: i64,
			aggregate_id: String,
			aggregate_name:String,
			topic: String,
			state: String
		);
		sqlx::query(
			r#"
            INSERT INTO service_outbox
                (id, aggregate_id, topic, state, aggregate_name)
            SELECT * FROM UNNEST
                ($1::BIGINT[], $2::text[],  $3::text[], $4::text[], $5::text[])
            "#,
		)
		.bind(&id)
		.bind(&aggregate_id)
		.bind(&topic)
		.bind(&state)
		.bind(&aggregate_name)
		.execute(self.executor.transaction())
		.await
		.map_err(|err| {
			tracing::error!("failed to insert outbox! {}", err);
			BaseError::DatabaseError(err.to_string())
		})?;
		Ok(())
	}
}

impl TRepository for SqlRepository {
	fn set_events(&mut self, events: VecDeque<std::sync::Arc<dyn TEvent>>) {
		self.events.extend(events)
	}
}

impl TUnitOfWork for SqlRepository {
	async fn begin(&mut self) -> Result<(), BaseError> {
		self.executor.begin().await
	}

	async fn _commit(&mut self) -> Result<(), BaseError> {
		self.executor.commit().await
	}

	async fn rollback(&mut self) -> Result<(), BaseError> {
		self.events.clear();
		self.executor.rollback().await
	}
	async fn close(&mut self) {
		self.executor.close().await;
	}

	async fn process_internal_events(&mut self) -> Result<(), BaseError> {
		self.send_internally_notifiable_messages().await;
		Ok(())
	}

	async fn process_external_events(&mut self) -> Result<(), BaseError> {
		self.save_outbox().await?;
		Ok(())
	}
}
