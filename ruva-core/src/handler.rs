use async_trait::async_trait;

use crate::prelude::{ApplicationError, ApplicationResponse, TCommand, TEvent};

pub trait TCommandService<R, E, C>: Send + Sync
where
	R: ApplicationResponse,
	E: ApplicationError,
	C: TCommand,
{
	fn execute(&mut self, cmd: C) -> impl std::future::Future<Output = Result<R, E>> + Send;
}

#[async_trait]
pub trait TEventService<E>: Send + Sync
where
	E: ApplicationError + Send + Sync,
{
	async fn execute(&mut self, cmd: std::sync::Arc<dyn TEvent>) -> Result<(), E>;
}
