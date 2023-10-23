use crate::prelude::{Aggregate, BaseError, Executor, Message};

use async_trait::async_trait;
use std::collections::VecDeque;

#[async_trait]
pub trait TRepository<E: Executor, A: Aggregate>: REventManager<A> + Sync + Send {
	async fn get(&self, aggregate_id: A::Identifier) -> Result<A, BaseError>;
	async fn update(&mut self, aggregate: &mut A) -> Result<(), BaseError>;
	async fn add(&mut self, aggregate: &mut A) -> Result<A::Identifier, BaseError>;
	async fn delete(&self, _aggregate_id: A::Identifier) -> Result<(), BaseError>;
}

pub trait REventManager<A: Aggregate> {
	fn get_events(&mut self) -> VecDeque<Box<dyn Message>>;
	fn set_events(&mut self, events: VecDeque<Box<dyn Message>>);
	fn event_hook(&mut self, aggregate: &mut A) {
		self.set_events(aggregate.take_events());
	}
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

#[async_trait]
pub trait TQueueProducer {
	async fn add(&self, payload: Box<dyn Message>) -> Result<(), BaseError>;
}
