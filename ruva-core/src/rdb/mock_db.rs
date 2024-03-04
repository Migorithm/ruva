use std::marker::PhantomData;

use crate::prelude::{AtomicContextManager, BaseError, OutBox, TAggregate, TCommitHook, TEvent, TRepository, TUnitOfWork};

pub fn outbox_table() -> &'static std::sync::Arc<tokio::sync::RwLock<Vec<OutBox>>> {
	static GROUP_TABLE: std::sync::OnceLock<std::sync::Arc<tokio::sync::RwLock<Vec<OutBox>>>> = std::sync::OnceLock::new();
	GROUP_TABLE.get_or_init(|| std::sync::Arc::new(tokio::sync::RwLock::new(vec![])))
}

#[derive(Clone)]
pub struct MockDb<A: TAggregate> {
	_aggregate: PhantomData<A>,
	pub events: std::collections::VecDeque<std::sync::Arc<dyn TEvent>>,
	pub context: AtomicContextManager,
}

impl<A: TAggregate> MockDb<A> {
	pub fn new(context: AtomicContextManager) -> Self {
		MockDb {
			_aggregate: PhantomData,
			events: Default::default(),
			context,
		}
	}

	pub(crate) async fn send_internally_notifiable_messages(&self) {
		let cxt = self.context.clone();
		let event_queue = &mut cxt.write().await;

		self.events.iter().filter(|e| e.internally_notifiable()).for_each(|e| event_queue.push_back(e.clone()));
	}

	pub(crate) async fn save_outbox(&self) -> Result<(), BaseError> {
		let outboxes = self.events.iter().filter(|e| e.externally_notifiable()).map(|o| o.outbox()).collect::<Vec<_>>();
		outbox_table().write().await.extend(outboxes);
		Ok(())
	}
}

impl<A: TAggregate> TRepository for MockDb<A> {
	fn set_events(&mut self, events: std::collections::VecDeque<std::sync::Arc<dyn TEvent>>) {
		self.events.extend(events)
	}
}
impl<A: TAggregate> TCommitHook for MockDb<A> {
	async fn commit_hook(&mut self) -> Result<(), crate::prelude::BaseError> {
		self.save_outbox().await?;
		self.send_internally_notifiable_messages().await;
		Ok(())
	}
}

impl<A: TAggregate> TUnitOfWork for MockDb<A> {
	async fn begin(&mut self) -> Result<(), crate::prelude::BaseError> {
		Ok(())
	}

	async fn commit(&mut self) -> Result<(), crate::prelude::BaseError> {
		self.commit_hook().await?;
		Ok(())
	}

	async fn rollback(&mut self) -> Result<(), crate::prelude::BaseError> {
		Ok(())
	}

	async fn close(&mut self) {}
}
