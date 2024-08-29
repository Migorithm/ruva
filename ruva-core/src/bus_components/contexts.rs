use super::executor::TConnection;
use crate::{make_smart_pointer, prelude::TEvent};
use std::{collections::VecDeque, sync::Arc};

/// Request Context Manager
/// it lives as long as the request lives

pub struct ContextManager {
	pub event_queue: VecDeque<Arc<dyn TEvent>>,
	pub conn: &'static dyn TConnection,
}

pub type AtomicContextManager = Arc<ContextManager>;

impl ContextManager {
	/// Creation of context manager returns context manager AND event receiver
	pub fn new(conn: &'static dyn TConnection) -> Self {
		Self { event_queue: VecDeque::new(), conn }
	}

	/// SAFETY: This is safe because we are sure this method is used only in the context of command and event handling
	pub(crate) fn get_mut<'a>(self: &Arc<Self>) -> &'a mut ContextManager {
		unsafe { &mut *(Arc::as_ptr(self) as *mut ContextManager) }
	}
}

make_smart_pointer!(ContextManager, VecDeque<Arc<dyn TEvent>>, event_queue);

/// Local context
/// it lasts only until logical unit of operation is done
pub struct Context {
	pub(crate) curr_events: VecDeque<std::sync::Arc<dyn TEvent>>,
	pub(crate) super_ctx: AtomicContextManager,

	#[cfg(feature = "sqlx-postgres")]
	pub(crate) pg_transaction: Option<sqlx::Transaction<'static, sqlx::Postgres>>,
}

impl Context {
	pub fn new(super_ctx: AtomicContextManager) -> Self {
		Self {
			curr_events: Default::default(),
			super_ctx,
			#[cfg(feature = "sqlx-postgres")]
			pg_transaction: None,
		}
	}

	pub fn event_hook(&mut self, aggregate: &mut impl crate::prelude::TAggregate) {
		self.set_current_events(aggregate.take_events());
	}

	pub async fn send_internally_notifiable_messages(&mut self) {
		// SAFETY: This is safe because we are sure that the context manager is not dropped

		self.curr_events
			.iter()
			.filter(|e| e.internally_notifiable())
			.for_each(|e| self.super_ctx.get_mut().push_back(e.clone()));
	}
}

pub trait TSetCurrentEvents: Send + Sync {
	fn set_current_events(&mut self, events: VecDeque<std::sync::Arc<dyn TEvent>>);
}

impl TSetCurrentEvents for Context {
	fn set_current_events(&mut self, events: VecDeque<std::sync::Arc<dyn TEvent>>) {
		self.curr_events.extend(events)
	}
}
