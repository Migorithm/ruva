use crate::prelude::{TCommitHook, TEvent};

use async_trait::async_trait;
use std::collections::VecDeque;

#[async_trait]
pub trait TRepository: Send + Sync + TCommitHook {
	fn set_events(&mut self, events: VecDeque<std::sync::Arc<dyn TEvent>>);
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
