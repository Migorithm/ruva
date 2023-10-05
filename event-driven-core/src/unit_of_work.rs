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
//! #### Usage Pattern 1
//!
//! ```ignore
//! // Intialize Uow, start transaction
//! let mut uow = UnitOfWork::<Repository<TaskAggregate>, TExecutor>::new(context).await;
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
//! #### Usage Pattern 2
//! Sometimes, you have to get the data from different aggregate and apply changes to
//! different aggregates. For that, you can switch repository and use the following pattern.
//!
//! ```ignore
//! // Intialize Uow, start transaction
//! let mut uow = UnitOfWork::<Repository<TaskAggregate>, TExecutor>::new(context).await;
//!
//! // Fetch data
//! let mut aggregate = uow.repository().get(&cmd.aggregate_id).await?;
//!
//! // Switch repo
//! let mut uow = uow.switch_repository::<Repository<DifferentTaskAggregate>>();
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
	prelude::{Aggregate, AtomicContextManager, BaseError, ContextManager},
	repository::TRepository,
};
use async_trait::async_trait;
use std::{marker::PhantomData, sync::Arc};
use tokio::sync::RwLock;

/// Executor is abstract implementation of whatever storage layer you use.
/// Among examples are RDBMS, Queue, NoSQLs.
#[async_trait]
pub trait Executor {
	async fn new() -> Arc<RwLock<Self>>;
	async fn begin(&mut self) -> Result<(), BaseError>;
	async fn commit(&mut self) -> Result<(), BaseError>;
	async fn rollback(&mut self) -> Result<(), BaseError>;
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

impl<R, E, A> UnitOfWork<R, E, A>
where
	R: TRepository<E, A>,
	E: Executor,
	A: Aggregate,
{
	pub fn clone_context(&self) -> AtomicContextManager {
		Arc::clone(&self.context)
	}

	/// Creeate UOW object with context manager.
	pub async fn new(context: AtomicContextManager) -> Self {
		let executor: Arc<RwLock<E>> = E::new().await;

		let mut uow = Self {
			repository: R::new(Arc::clone(&executor)),
			context,
			executor,
			_aggregate: Default::default(),
		};
		uow.begin().await.unwrap();
		uow
	}

	/// Switch repository to different type.
	///
	/// \#\#\# Example
	/// ```ignore
	/// let mut uow = UnitOfWork::<A<Executer>, Executor>::new(context).await;
	/// let new_uow = uow.switch_repository::<B<Executor>>(); // origin uow was deleted.
	/// ```
	pub fn switch_repository<DR, DA>(mut self) -> UnitOfWork<DR, E, DA>
	where
		DR: TRepository<E, DA>,
		DA: Aggregate,
	{
		let mut repo = DR::new(Arc::clone(&self.executor));
		repo.set_events(self.repository().get_events());

		UnitOfWork {
			executor: Arc::clone(&self.executor),
			context: self.context,
			repository: repo,
			_aggregate: Default::default(),
		}
	}

	/// Get local event repository.
	pub fn repository(&mut self) -> &mut R {
		&mut self.repository
	}

	/// Begin transaction.
	async fn begin(&mut self) -> Result<(), BaseError> {
		let mut executor = self.executor.write().await;
		executor.begin().await
	}

	/// Get inner executor.
	fn executor(&self) -> Arc<RwLock<E>> {
		Arc::clone(&self.executor)
	}

	/// Commit transaction.
	pub async fn commit<O: IOutBox<E>>(mut self) -> Result<(), BaseError> {
		// To drop uow itself!

		// run commit hook
		self._commit_hook::<O>().await?;

		// commit
		self._commit().await
	}

	async fn _commit(&mut self) -> Result<(), BaseError> {
		let mut executor = self.executor.write().await;

		executor.commit().await
	}

	/// Rollback transaction.
	pub async fn rollback(self) -> Result<(), BaseError> {
		let mut executor = self.executor.write().await;
		executor.rollback().await
	}

	/// commit_hook is invoked right before the calling for commit
	/// which sorts out and processes outboxes and internally processable events.
	async fn _commit_hook<O: IOutBox<E>>(&mut self) -> Result<(), BaseError> {
		let event_queue = &mut self.context.write().await;
		let mut outboxes = vec![];

		for e in self.repository.get_events() {
			if e.externally_notifiable() {
				outboxes.push(e.outbox());
			};
			if e.internally_notifiable() {
				event_queue.push_back(e.message_clone());
			}
		}
		O::add(self.executor(), outboxes).await
	}
}
