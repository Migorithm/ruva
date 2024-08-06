//! ### example
//! ```rust,no_run
//! impl<R, C> TCommandService<ServiceResponse, ServiceError> for UnitOfWorkHandler<C, R>
//! where
//!     R: TRepository + TUnitOfWork,
//!     C: TCommand,
//!     Self: TRunCommand,
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

pub mod event;

use crate::prelude::{ApplicationError, ApplicationResponse};
pub use event::*;

pub trait TCommandService<R, E>: Send + Sync
where
	R: ApplicationResponse,
	E: ApplicationError,
{
	fn execute(&mut self) -> impl std::future::Future<Output = Result<R, E>> + Send;
}
