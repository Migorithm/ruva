use crate::prelude::TEvent;
use std::{
	collections::VecDeque,
	ops::{Deref, DerefMut},
	sync::Arc,
};
use tokio::sync::RwLock;

pub type AtomicContextManager = Arc<RwLock<ContextManager>>;

/// Task Local Context Manager
/// This is called for every time `handle` method is invoked.
pub struct ContextManager {
	pub event_queue: VecDeque<Arc<dyn TEvent>>,
}

impl ContextManager {
	/// Creation of context manager returns context manager AND event receiver
	pub fn new() -> AtomicContextManager {
		Arc::new(RwLock::new(Self { event_queue: VecDeque::new() }))
	}
}

impl Deref for ContextManager {
	type Target = VecDeque<Arc<dyn TEvent>>;
	fn deref(&self) -> &Self::Target {
		&self.event_queue
	}
}
impl DerefMut for ContextManager {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.event_queue
	}
}
