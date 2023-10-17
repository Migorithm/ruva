//! ### UnitOfWork
//! [UnitOfWork][UOW] is to a unit that manages atomic transaction.
//!
//! Its [executor][Exec] is supposed to be shared with its sub type [Repository][TRepository].
//!
//! `commit`, and `rollback`, is governed by this implementation.
//!
//! When events are collected in `Repository`[TRepository], you can collect them
//!
//! automatically thanks to `_commit_hook` method.
//!
//! [UOW]: crate::unit_of_work::UnitOfWork
//! [TRepository]: crate::repository::TRepository
//! [Exec]: crate::unit_of_work::Executor
//! [Handler]: crate::unit_of_work::Handler
//!
//! #### Usage Pattern
//!
//! ```ignore
//! // Intialize Uow, start transaction
//! let mut uow = UnitOfWork::<Repository<TaskAggregate>, Executor,TaskAggregate>::new(context).await;
//!
//! // Fetch data
//! let mut aggregate = uow.repository().get(&cmd.aggregate_id).await?;
//!
//! // Process business logic
//! aggregate.process_business_logic(cmd)?;
//!
//! // Apply changes
//! uow.repository().update(&mut aggregate).await?;
//!
//! // Commit transaction
//! uow.commit::<ServiceOutBox>().await?;
//! ```
//!
//!
//!
//! ### Handler
//! [Handler] is what orchestrates operations from data fetching, business logic operation and store
//! changes back to db. This is where tranasction occurs.
//!
//! ### Example
//! ```ignore
//! struct ApplicationHandler;
//! impl Handler for ApplicationHandler{
//!     type E = ApplicationExecutor;
//!     type R = ApplicationRepository<Aggregate>
//! }
//!
//! impl ApplicationHandler{
//!     pub async fn serve_request(
//!         cmd: Command1,
//!         context: AtomicContextManager,
//! ) -> Result<(),ServiceError> {
//!     let mut uow = TaskHandler::uow(context).await;
//! }
//! ```

use crate::{
	outbox::IOutBox,
	prelude::{Aggregate, AtomicContextManager, BaseError},
	repository::TRepository,
};
use async_trait::async_trait;
use std::{marker::PhantomData, sync::Arc};
use tokio::sync::RwLock;

/// Executor is abstract implementation of whatever storage layer you use.
/// Among examples are RDBMS, Queue, NoSQLs.
#[async_trait]
pub trait Executor: Sync + Send {
	async fn new() -> Arc<RwLock<Self>>;
	async fn begin(&mut self) -> Result<(), BaseError>;
	async fn commit(&mut self) -> Result<(), BaseError>;
	async fn rollback(&mut self) -> Result<(), BaseError>;
}

#[async_trait]
pub trait TUnitOfWork<R, E, A>: Send + Sync
where
	R: TRepository<E, A>,
	E: Executor,
	A: Aggregate,
{
	fn context(&self) -> AtomicContextManager;
	fn executor(&self) -> Arc<RwLock<E>>;

	/// Creeate UOW object with context manager.
	fn repository(&mut self) -> &mut R;

	async fn begin(&mut self) -> Result<(), BaseError> {
		self.executor().write().await.begin().await
	}

	async fn commit<O: IOutBox<E>>(&mut self) -> Result<(), BaseError> {
		//* Hook - sending event
		let binding = self.context();
		let event_queue = &mut binding.write().await;
		let mut outboxes = vec![];

		for e in self.repository().get_events() {
			if e.externally_notifiable() {
				outboxes.push(e.outbox());
			};
			if e.internally_notifiable() {
				event_queue.push_back(e.message_clone());
			}
		}
		O::add(self.executor(), outboxes).await?;

		//* */ commit operation
		self.executor().write().await.commit().await
	}
	async fn rollback(&mut self) -> Result<(), BaseError> {
		self.executor().write().await.rollback().await
	}
}

#[derive(Clone)]
pub struct UnitOfWork<R, E, A>
where
	R: TRepository<E, A>,
	E: Executor,
	A: Aggregate,
{
	/// real transaction executor
	executor: Arc<RwLock<E>>,
	/// global event event_queue
	context: AtomicContextManager,
	_aggregate: PhantomData<A>,

	/// event local repository for Executor
	pub repository: R,
}

// TODO new, begin sepration
#[async_trait]
impl<R, E, A> TUnitOfWork<R, E, A> for UnitOfWork<R, E, A>
where
	R: TRepository<E, A>,
	E: Executor,
	A: Aggregate,
{
	fn context(&self) -> AtomicContextManager {
		Arc::clone(&self.context)
	}
	fn executor(&self) -> Arc<RwLock<E>> {
		self.executor.clone()
	}

	/// Get local event repository.
	fn repository(&mut self) -> &mut R {
		&mut self.repository
	}
}

impl<R, E, A> UnitOfWork<R, E, A>
where
	R: TRepository<E, A>,
	E: Executor,
	A: Aggregate,
{
	pub async fn new(context: AtomicContextManager) -> Self {
		let executor: Arc<RwLock<E>> = E::new().await;

		let mut uow = Self {
			repository: R::new(Arc::clone(&executor)),
			context,
			executor,
			_aggregate: PhantomData,
		};
		uow.begin().await.unwrap();
		uow
	}
}
