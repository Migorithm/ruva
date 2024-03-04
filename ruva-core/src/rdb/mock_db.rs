use std::marker::PhantomData;

use crate::prelude::{AtomicContextManager, TAggregate, TCommitHook, TEvent, TRepository, TUnitOfWork};

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
}

impl<A: TAggregate> TRepository for MockDb<A> {
	fn set_events(&mut self, events: std::collections::VecDeque<std::sync::Arc<dyn TEvent>>) {
		self.events.extend(events)
	}
}
impl<A: TAggregate> TCommitHook for MockDb<A> {
	async fn commit_hook(&mut self) -> Result<(), crate::prelude::BaseError> {
		Ok(())
	}
}

impl<A: TAggregate> TUnitOfWork for MockDb<A> {
	async fn begin(&mut self) -> Result<(), crate::prelude::BaseError> {
		Ok(())
	}

	async fn commit(&mut self) -> Result<(), crate::prelude::BaseError> {
		Ok(())
	}

	async fn rollback(&mut self) -> Result<(), crate::prelude::BaseError> {
		Ok(())
	}

	async fn close(&mut self) {}
}
