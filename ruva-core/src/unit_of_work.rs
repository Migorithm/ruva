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
//! [Exec]: crate::unit_of_work::TExecutor
//! [Handler]: crate::unit_of_work::Handler
//!
//! #### Usage Pattern
//!
//! ```ignore
//! // Intialize Uow, start transaction
//! let mut uow = UnitOfWork::<Repository<TaskAggregate>, TExecutor,TaskAggregate>::new(context).await;
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
//!     type R = ApplicationRepository<TAggregate>
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
	prelude::{BaseError, TCloneContext},
	repository::TRepository,
};
use async_trait::async_trait;

#[async_trait]
pub trait TUnitOfWork: Send + Sync {
	/// Creeate UOW object with context manager.

	async fn begin(&mut self) -> Result<(), BaseError>;

	async fn commit(&mut self) -> Result<(), BaseError>;

	async fn rollback(&mut self) -> Result<(), BaseError>;
}

#[async_trait]
pub trait TCommitHook: TRepository + TCloneContext {
	async fn commit_hook(&mut self) -> Result<(), BaseError> {
		let cxt = self.clone_context();
		let event_queue = &mut cxt.write().await;
		let mut outboxes = vec![];

		for e in self.get_events() {
			if e.externally_notifiable() {
				outboxes.push(e.outbox());
			};
			if e.internally_notifiable() {
				event_queue.push_back(e.clone());
			}
		}
		if !outboxes.is_empty() {
			self.save_outbox(outboxes).await;
		}
		Ok(())
	}
}
