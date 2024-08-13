use crate::prelude::{AtomicContextManager, TEvent};

use std::collections::VecDeque;

pub trait TRepository: Send + Sync {
	fn set_events(&mut self, events: VecDeque<std::sync::Arc<dyn TEvent>>);
}

pub struct Context {
	pub(crate) events: VecDeque<std::sync::Arc<dyn TEvent>>,
	pub(crate) inner: AtomicContextManager,

	#[cfg(feature = "sqlx-postgres")]
	pub(crate) pg_transaction: Option<sqlx::Transaction<'static, sqlx::Postgres>>,
}

impl Context {
	pub fn new(context: AtomicContextManager) -> Self {
		Self {
			events: Default::default(),
			inner: context,
			#[cfg(feature = "sqlx-postgres")]
			pg_transaction: None,
		}
	}

	pub fn event_hook(&mut self, aggregate: &mut impl crate::prelude::TAggregate) {
		self.set_events(aggregate.take_events());
	}

	pub async fn send_internally_notifiable_messages(&self) {
		let event_queue = &mut self.inner.write().await;

		self.events.iter().filter(|e| e.internally_notifiable()).for_each(|e| event_queue.push_back(e.clone()));
	}
}

impl TRepository for Context {
	fn set_events(&mut self, events: VecDeque<std::sync::Arc<dyn TEvent>>) {
		self.events.extend(events)
	}
}

// To Support Bulk Insert Operation
#[macro_export]
macro_rules! prepare_bulk_operation {
    (
        $subject:expr, $($field:ident:$field_type:ty),*
    ) => {
        $(
            let mut $field:Vec<$field_type> = Vec::with_capacity($subject.len());
        )*

        $subject.iter().for_each(|subj|{
            $(
                $field.push(subj.$field.clone());
            )*
        }
        )

    };
    (
        $subject:expr, $($field:ident():$field_type:ty),*
    ) =>{
        $(
            let mut $field:Vec<$field_type> = Vec::with_capacity($subject.len());
        )*

        $subject.iter().for_each(|subj|{
            $(
                $field.push(subj.$field().to_owned());
            )*
        }
        )
    }
}
