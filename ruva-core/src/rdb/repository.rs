use std::{collections::VecDeque, marker::PhantomData, sync::Arc};

use tokio::sync::RwLock;

use crate::{
	prelude::{AtomicContextManager, BaseError, TAggregate, TClone, TCloneContext, TCommitHook, TEvent, TUnitOfWork},
	prepare_bulk_operation,
	repository::TRepository,
};

use super::executor::SQLExecutor;

pub struct SqlRepository<A: TAggregate> {
	pub executor: Arc<RwLock<SQLExecutor>>,
	_phantom: PhantomData<A>,
	events: VecDeque<std::sync::Arc<dyn TEvent>>,
	context: AtomicContextManager,
}

impl<A: TAggregate + 'static> SqlRepository<A> {
	pub fn new(context: AtomicContextManager, executor: Arc<RwLock<SQLExecutor>>) -> Self {
		Self {
			executor,
			_phantom: std::marker::PhantomData,
			events: Default::default(),
			context,
		}
	}
	pub fn event_hook(&mut self, aggregate: &mut A) {
		self.set_events(aggregate.take_events());
	}
	pub(crate) async fn send_internally_notifiable_messages(&self) {
		let cxt = self.clone_context();
		let event_queue = &mut cxt.write().await;

		self.events.iter().filter(|e| e.internally_notifiable()).for_each(|e| event_queue.push_back(e.clone()));
	}
	pub(crate) async fn save_outbox(&self) -> Result<(), BaseError> {
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
		.execute(self.executor.write().await.transaction())
		.await
		.map_err(|err| {
			tracing::error!("failed to insert outbox! {}", err);
			BaseError::DatabaseError(Box::new(err))
		})?;
		Ok(())
	}
}

impl<A: TAggregate> TClone for SqlRepository<A> {
	fn clone(&self) -> Self {
		Self {
			executor: self.executor.clone(),
			_phantom: PhantomData,
			events: Default::default(),
			context: self.context.clone(),
		}
	}
}

impl<A: TAggregate> TCloneContext for SqlRepository<A> {
	fn clone_context(&self) -> AtomicContextManager {
		self.context.clone()
	}
}

impl<A: TAggregate + 'static> TRepository for SqlRepository<A> {
	fn set_events(&mut self, events: VecDeque<std::sync::Arc<dyn TEvent>>) {
		self.events.extend(events)
	}
}

impl<A: TAggregate + 'static> TCommitHook for SqlRepository<A> {
	async fn commit_hook(&mut self) -> Result<(), BaseError> {
		self.save_outbox().await?;
		self.send_internally_notifiable_messages().await;
		Ok(())
	}
}

impl<A: TAggregate + 'static> TUnitOfWork for SqlRepository<A> {
	async fn begin(&mut self) -> Result<(), BaseError> {
		let mut executor = self.executor.write().await;
		executor.begin().await
	}

	async fn commit(&mut self) -> Result<(), BaseError> {
		// run commit hook
		self.commit_hook().await?;

		// commit
		self.executor.write().await.commit().await
	}

	async fn rollback(&mut self) -> Result<(), BaseError> {
		let mut executor = self.executor.write().await;
		executor.rollback().await
	}
	async fn close(&self) {
		self.executor.read().await.close().await;
	}
}
