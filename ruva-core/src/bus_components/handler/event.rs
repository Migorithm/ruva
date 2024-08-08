use crate::prelude::TEvent;

use std::pin::Pin;

use crate::bus_components::contexts::AtomicContextManager;

pub type Future<E> = Pin<Box<dyn futures::Future<Output = Result<(), E>> + Send>>;
pub type FutureResult<E> = Result<Future<E>, E>;

pub type Handlers<E> = Vec<Box<dyn Fn(std::sync::Arc<dyn TEvent>, AtomicContextManager) -> Future<E> + Send + Sync>>;

pub enum EventHandlers<E> {
	Sync(Handlers<E>),
	Async(Handlers<E>),
}
impl<E> EventHandlers<E> {
	pub fn extend(&mut self, handlers: Handlers<E>) {
		match self {
			Self::Sync(h) => h.extend(handlers),
			Self::Async(h) => h.extend(handlers),
		}
	}
}
