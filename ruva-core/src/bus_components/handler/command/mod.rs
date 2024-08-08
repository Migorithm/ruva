//! ### Example - simple command handler
//! ```rust,no_run
//! impl<C,R> TCommandService<(), ()> for CommandHandler<(C, R)>
//! where
//!     C: crate::prelude::TCommand + for<'a> TGetHandler<&'a mut R, Result<(), ()>>,
//!     R: Send + Sync,
//! {
//!     async fn execute(mut self) -> Result<(), ()> {
//!         let CommandHandler((cmd, mut dep)) = self;
//!         let handler = C::get_handler();
//!         handler(cmd, &mut dep).await
//!     }
//! }
//! ```

pub mod uow;
use crate::{
	message::TCommand,
	prelude::{ApplicationError, ApplicationResponse, BaseError, TCommandService, TRepository, TUnitOfWork},
};
pub use uow::*;

pub struct CommandHandler<T>(pub T);

impl<T> CommandHandler<T> {
	pub fn destruct(self) -> T {
		self.0
	}
}

pub trait AsyncFunc<C, R, ApplicationResult>: Fn(C, R) -> Self::Fut + Send + Sync {
	type Fut: std::future::Future<Output = ApplicationResult> + Send;
}

impl<F, C, Fut, Respository, ApplicationResult> AsyncFunc<C, Respository, ApplicationResult> for F
where
	C: crate::prelude::TCommand,
	F: Fn(C, Respository) -> Fut + Send + Sync,
	Fut: std::future::Future<Output = ApplicationResult> + Send,
{
	type Fut = Fut;
}

pub trait TGetHandler<R, ApplicationResult>: Sized {
	fn get_handler() -> impl AsyncFunc<Self, R, ApplicationResult>;
}
