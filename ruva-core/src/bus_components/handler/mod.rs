//!
//!
//! ### example - transaction unit of work
//! ```rust,no_run
//! impl<C, R> TCommandService<ServiceResponse, ServiceError> for CommandHandler<(C, R)>
//! where
//!     C: TCommand + for<'a> TGetHandler<&'a mut R, Result<ServiceResponse>>,
//!     R: TSetCurrentEvents + TUnitOfWork,
//! {
//!     async fn execute(&mut self) -> Result<ServiceResponse> {
//!         let CommandHandler((cmd, dep)) = self;
//!         dep.begin().await?;
//!
//!         match (C::get_handler())(cmd, &mut dep).await {
//!             Ok(val) => {
//!                 dep.commit().await?;
//!                 dep.close().await;
//!
//!                 Ok(val)
//!             }
//!             Err(err) => {
//!                 dep.rollback().await?;
//!                 dep.close().await;
//!                 if let ServiceError::StopSentinelWithEvent(event) = err {
//!                     dep.set_events(vec![event.clone()].into());
//!                     dep.process_internal_events().await?;
//!                     dep.process_external_events().await?;
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

pub mod command;

pub mod event;
pub use command::*;
pub use event::*;
