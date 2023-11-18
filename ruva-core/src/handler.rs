use async_trait::async_trait;

use crate::prelude::{ApplicationError, ApplicationResponse, TCommand};

#[async_trait]
pub trait TCommandService<R, E, C>: Send + Sync
where
	R: ApplicationResponse,
	E: ApplicationError,
	C: TCommand,
{
	async fn execute(&mut self, cmd: C) -> Result<R, E>;
}
