use std::{collections::VecDeque, sync::Arc};

use async_trait::async_trait;

use tokio::sync::RwLock;

use crate::{domain::Message, responses::BaseError};

#[async_trait]
pub trait Executor: Send + Sync + 'static {
    async fn begin(&mut self) -> Result<(), BaseError>;
    async fn commit(&mut self) -> Result<(), BaseError>;
    async fn rollback(&mut self) -> Result<(), BaseError>;
}

pub trait TRepository<E: Executor> {
    fn new(executor: Arc<RwLock<E>>) -> Self;

    fn get_events(&mut self) -> VecDeque<Box<dyn Message>>;
    fn set_events(&mut self, events: VecDeque<Box<dyn Message>>);
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
