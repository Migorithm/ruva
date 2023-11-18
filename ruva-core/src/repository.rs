use crate::prelude::{OutBox, TAggregate, TEvent};

use async_trait::async_trait;
use std::collections::VecDeque;

#[async_trait]
pub trait TRepository: Send + Sync {
	fn get_events(&mut self) -> VecDeque<std::sync::Arc<dyn TEvent>>;
	fn set_events(&mut self, events: VecDeque<std::sync::Arc<dyn TEvent>>);
	fn event_hook<A: TAggregate>(&mut self, aggregate: &mut A) {
		self.set_events(aggregate.take_events());
	}
	async fn save_outbox(&mut self, outboxes: Vec<OutBox>);
}

pub trait TRepositoyCallable<R>
where
	R: TRepository,
{
	fn repository(&mut self) -> &mut R;
}

// To Support Bulk Insert Operation
#[macro_export]
macro_rules! prepare_bulk_insert {
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
