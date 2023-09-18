use crate::prelude::{Aggregate, Executor, Message};

use std::{collections::VecDeque, sync::Arc};
use tokio::sync::RwLock;

pub trait TRepository<E: Executor, A: Aggregate>: REventManager<A> {
	fn new(executor: Arc<RwLock<E>>) -> Self;
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
