use crate::prelude::{ApplicationError, ApplicationResponse, TCommand, TEvent};
use hashbrown::HashMap;
use std::pin::Pin;

use super::contexts::AtomicContextManager;

pub trait TCommandService<R, E, C>: Send + Sync
where
	R: ApplicationResponse,
	E: ApplicationError,
	C: TCommand,
{
	fn execute(&mut self, cmd: C) -> impl std::future::Future<Output = Result<R, E>> + Send;
}

pub type TEventHandler<R, E> = HashMap<String, EventHandlers<R, E>>;
pub type Handlers<R, E> = Vec<Box<dyn Fn(std::sync::Arc<dyn TEvent>, AtomicContextManager) -> Future<R, E> + Send + Sync>>;
pub type Future<T, E> = Pin<Box<dyn futures::Future<Output = Result<T, E>> + Send>>;
pub enum EventHandlers<R, E> {
	Sync(Handlers<R, E>),
	Async(Handlers<R, E>),
}
impl<R, E> EventHandlers<R, E> {
	pub fn extend(&mut self, handlers: Handlers<R, E>) {
		match self {
			Self::Sync(h) => h.extend(handlers),
			Self::Async(h) => h.extend(handlers),
		}
	}
}
