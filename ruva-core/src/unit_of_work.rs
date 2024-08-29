//! ### TUnitOfWork
//! [TUnitOfWork] is a trait to manage atomic transaction.
//!
//! `commit`, and `rollback`, is governed by this implementation.
//!
//!
//! To make sure that events raised in `Aggregate` object is properly collected, you want to implement
//!
//! [TCommitHook] as well.
//!
//!
//! [UOW]: crate::unit_of_work::TUnitOfWork
//! [TCommitHook]: crate::unit_of_work::TCommitHook

//! [Handler]: crate::unit_of_work::Handler
//!
//! #### Usage Pattern
//!
//! ```rust,no_run
//! // Service Handler
//! pub struct CustomHandler<R> {
//!     _r: PhantomData<R>,
//! }
//! impl<R> CustomHandler<R>
//! where
//!     R: TCustomRepository + TUnitOfWork,
//! {
//!     pub async fn create_aggregate(
//!         cmd: CreateCommand,
//!         mut uow: R,
//!     ) -> Result<CustomResponse, CustomError> {
//!         // Transation begin
//!         uow.begin().await?;
//!         let mut aggregate: CustomAggregate = CustomAggregate::new(cmd);
//!         uow.add(&mut aggregate).await?;
//!
//!         // Transation commit
//!         uow.commit().await?;
//!         Ok(aggregate.id.into())
//!     }
//! }
//!
//! ```
//!

use crate::prelude::BaseError;

/// Template for Unit of Work
/// Concrete implementation must implement `_commit` method
/// If you want to add hooks on events, you can implement `process_internal_events` and `process_external_events`

pub trait TUnitOfWork: Send + Sync {
	fn begin(&mut self) -> impl std::future::Future<Output = Result<(), BaseError>> + Send;

	// Template method
	fn commit(&mut self) -> impl std::future::Future<Output = Result<(), BaseError>> + Send {
		async {
			self.process_internal_events().await?;
			self.process_external_events().await?;
			self._commit().await?;
			Ok(())
		}
	}
	// Actual commit which concrete implementation must implement
	fn _commit(&mut self) -> impl std::future::Future<Output = Result<(), BaseError>> + Send;

	fn rollback(&mut self) -> impl std::future::Future<Output = Result<(), BaseError>> + Send;

	fn close(&mut self) -> impl std::future::Future<Output = ()> + Send;

	// Hook
	fn process_internal_events(&mut self) -> impl std::future::Future<Output = Result<(), BaseError>> + Send {
		async { Ok(()) }
	}
	// Hook
	fn process_external_events(&mut self) -> impl std::future::Future<Output = Result<(), BaseError>> + Send {
		async { Ok(()) }
	}
}
