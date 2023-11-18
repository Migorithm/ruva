use std::collections::VecDeque;

use crate::prelude::TEvent;

pub trait TAggregate: Send + Sync + Default {
	type Identifier: Send + Sync;
	fn collect_events(&mut self) -> VecDeque<std::sync::Arc<dyn TEvent>> {
		if !self.events().is_empty() {
			self.take_events()
		} else {
			VecDeque::new()
		}
	}
	fn events(&self) -> &std::collections::VecDeque<std::sync::Arc<dyn TEvent>>;

	fn take_events(&mut self) -> std::collections::VecDeque<std::sync::Arc<dyn TEvent>>;
	fn raise_event(&mut self, event: std::sync::Arc<dyn TEvent>);
}
