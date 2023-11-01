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
//! uow.commit().await?;
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
	prelude::{AtomicContextManager, BaseError},
	repository::TRepository,
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Executor is abstract implementation of whatever storage layer you use.
/// Among examples are RDBMS, Queue, NoSQLs.
#[async_trait]
pub trait Executor: Sync + Send {
	async fn begin(&mut self) -> Result<(), BaseError>;
	async fn commit(&mut self) -> Result<(), BaseError>;
	async fn rollback(&mut self) -> Result<(), BaseError>;
}

#[async_trait]
pub trait TUnitOfWork: Send + Sync {
	/// Creeate UOW object with context manager.

	async fn begin(&mut self) -> Result<(), BaseError>;

	async fn commit(&mut self) -> Result<(), BaseError>;

	async fn rollback(self) -> Result<(), BaseError>;
}

pub trait TRepositoyCallable<R>
where
	R: TRepository,
{
	fn repository(&mut self) -> &mut R;
}

#[derive(Clone)]
pub struct UnitOfWork<R, E>
where
	R: TRepository,
	E: Executor,
{
	/// real transaction executor
	executor: Arc<RwLock<E>>,
	/// global event event_queue
	context: AtomicContextManager,

	/// event local repository for Executor
	pub repository: R,
}
impl<R, E> UnitOfWork<R, E>
where
	R: TRepository,
	E: Executor,
{
	pub fn new(context: AtomicContextManager, executor: Arc<RwLock<E>>, repository: R) -> Self {
		Self { repository, context, executor }
	}
	async fn _commit(&mut self) -> Result<(), BaseError> {
		let mut executor = self.executor.write().await;

		executor.commit().await
	}
	/// commit_hook is invoked right before the calling for commit
	/// which sorts out and processes outboxes and internally processable events.
	async fn _commit_hook(&mut self) -> Result<(), BaseError> {
		let cxt = self.clone_context();
		let event_queue = &mut cxt.write().await;
		let mut outboxes = vec![];

		for e in self.repository.get_events() {
			if e.externally_notifiable() {
				outboxes.push(e.outbox());
			};
			if e.internally_notifiable() {
				event_queue.push_back(e.message_clone());
			}
		}
		if !outboxes.is_empty() {
			self.repository().save_outbox(outboxes).await;
		}
		Ok(())
	}
}

#[async_trait]
impl<R, E> TUnitOfWork for UnitOfWork<R, E>
where
	R: TRepository,
	E: Executor,
{
	/// Begin transaction.
	async fn begin(&mut self) -> Result<(), BaseError> {
		let mut executor = self.executor.write().await;
		executor.begin().await
	}

	/// Commit transaction.
	async fn commit(&mut self) -> Result<(), BaseError> {
		// To drop uow itself!

		// run commit hook
		self._commit_hook().await?;

		// commit
		self._commit().await
	}

	/// Rollback transaction.
	async fn rollback(self) -> Result<(), BaseError> {
		let mut executor = self.executor.write().await;
		executor.rollback().await
	}
}

impl<R, E> TRepositoyCallable<R> for UnitOfWork<R, E>
where
	R: TRepository,
	E: Executor,
{
	fn repository(&mut self) -> &mut R {
		&mut self.repository
	}
}

pub trait TCloneContext {
	fn clone_context(&self) -> AtomicContextManager;
}

impl<R, E> TCloneContext for UnitOfWork<R, E>
where
	R: TRepository,
	E: Executor,
{
	fn clone_context(&self) -> AtomicContextManager {
		Arc::clone(&self.context)
	}
}
