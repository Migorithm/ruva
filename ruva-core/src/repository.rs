use crate::prelude::{Aggregate, BaseError, Message};

use async_trait::async_trait;
use std::collections::VecDeque;

pub trait TRepository: Send + Sync {
	fn get_events(&mut self) -> VecDeque<Box<dyn Message>>;
	fn set_events(&mut self, events: VecDeque<Box<dyn Message>>);
	fn event_hook<A: Aggregate>(&mut self, aggregate: &mut A) {
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
	type HeaderMeta;
	async fn add(&self, header_meta: Self::HeaderMeta, payload: Box<dyn Message>) -> Result<(), BaseError>;
}
