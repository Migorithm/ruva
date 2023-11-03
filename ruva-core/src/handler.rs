use async_trait::async_trait;

use crate::prelude::{ApplicationError, ApplicationResponse, Command};

#[async_trait]
pub trait TCommandService<R, E, C>: Send + Sync
where
	R: ApplicationResponse,
	E: ApplicationError,
	C: Command,
{
	async fn execute(&mut self, cmd: C) -> Result<R, E>;
}
