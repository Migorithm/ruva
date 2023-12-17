//! ### TUnitOfWork
//! [TUnitOfWork] is a trait to manage atomic transaction.
//!
//! `commit`, and `rollback`, is governed by this implementation.
//!
//! Concrete implementation that implements [TRepository] may also implement [TUnitOfWork]
//!
//! To make sure that events raised in `Aggregate` object is properly collected, you want to implement
//!
//! [TCommitHook] as well.
//!
//!
//! [UOW]: crate::unit_of_work::TUnitOfWork
//! [TCommitHook]: crate::unit_of_work::TCommitHook
//! [TRepository]: crate::repository::TRepository
//! [Handler]: crate::unit_of_work::Handler
//!
//! #### Usage Pattern
//!
//! ```ignore
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
//! Note that if you don't "attatch" [TUnitOfWork], the `uow` above would only have an access to [TRepository] but not transation-related methods.
//!

use crate::prelude::BaseError;
use async_trait::async_trait;

#[async_trait]
pub trait TUnitOfWork: Send + Sync {
	/// Creeate UOW object with context manager.

	async fn begin(&mut self) -> Result<(), BaseError>;

	async fn commit(&mut self) -> Result<(), BaseError>;

	async fn rollback(&mut self) -> Result<(), BaseError>;
}

#[async_trait]
pub trait TCommitHook {
	async fn commit_hook(&mut self) -> Result<(), BaseError>;
}
