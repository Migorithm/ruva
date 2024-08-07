//! ### Example - simple command handler
//! ```rust,no_run
//! impl<R, C> TCommandService<(), ()> for CommandHandler<C, R>
//! where
//!     C: crate::prelude::TCommand + for<'a> TGetHandler<&'a mut R, Result<(), ()>>,
//!     R: Send + Sync,
//! {
//!     async fn execute(mut self) -> Result<(), ()> {
//!         let handler = C::get_handler();
//!         handler(self.cmd, &mut self.dep).await
//!     }
//! }
//! ```
//!
//!
//! ### example - transaction unit of work
//! ```rust,no_run
//! impl<R, C> TCommandService<ServiceResponse, ServiceError> for CommandHandler<C, R>
//! where
//!     R: TRepository + TUnitOfWork,
//!     C: TCommand + for<'a> TGetHandler<&'a mut R, Result<ServiceResponse>>,
//! {
//!     async fn execute(&mut self) -> Result<ServiceResponse> {
//!         self.begin().await?;
//!
//!         match self.run_command().await {
//!             Ok(val) => {
//!                 self.commit().await?;
//!                 self.close().await;
//!
//!                 Ok(val)
//!             }
//!             Err(err) => {
//!                 self.rollback().await?;
//!                 self.close().await;
//!                 if let ServiceError::StopSentinelWithEvent(event) = err {
//!                     self.set_events(vec![event.clone()].into());
//!                     self.process_internal_events().await?;
//!                     self.process_external_events().await?;
//!                     Err(ServiceError::StopSentinelWithEvent(event))
//!                 } else {
//!                     Err(err)
//!                 }
//!             }
//!         }
//!     }
//! }
//! ```
//!

pub mod event;
use crate::prelude::{ApplicationError, ApplicationResponse};
pub use event::*;

pub trait TCommandService<R, E>: Send + Sync
where
	R: ApplicationResponse,
	E: ApplicationError,
{
	fn execute(self) -> impl std::future::Future<Output = Result<R, E>> + Send;
}

pub trait AsyncFunc<Command, Respository, ApplicationResult>: Fn(Command, Respository) -> Self::Fut + Send + Sync {
	type Fut: std::future::Future<Output = ApplicationResult> + Send;
}

impl<F, Command, Fut, Respository, ApplicationResult> AsyncFunc<Command, Respository, ApplicationResult> for F
where
	Command: crate::prelude::TCommand,
	F: Fn(Command, Respository) -> Fut + Send + Sync,
	Fut: std::future::Future<Output = ApplicationResult> + Send,
{
	type Fut = Fut;
}

pub trait TGetHandler<Respository, ApplicationResult>: Sized {
	fn get_handler() -> impl AsyncFunc<Self, Respository, ApplicationResult>;
}

pub struct CommandHandler<C, R> {
	pub dep: R,
	pub cmd: C,
}

impl<C, R> CommandHandler<C, R> {
	pub fn new(dep: R, cmd: C) -> Self {
		Self { dep, cmd }
	}
}
