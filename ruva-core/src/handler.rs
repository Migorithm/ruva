use crate::prelude::{ApplicationError, ApplicationResponse, TCommand};

pub trait TCommandService<R, E, C>: Send + Sync
where
	R: ApplicationResponse,
	E: ApplicationError,
	C: TCommand,
{
	fn execute(&mut self, cmd: C) -> impl std::future::Future<Output = Result<R, E>> + Send;
}
