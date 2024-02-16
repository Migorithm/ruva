use tokio::runtime::Handle;

use crate::prelude::{TCommitHook, TEvent};

use std::collections::VecDeque;

pub trait TRepository: Send + Sync + TCommitHook {
	fn set_events(&mut self, events: VecDeque<std::sync::Arc<dyn TEvent>>);
}

impl<T: TRepository> TRepository for &mut T {
	fn set_events(&mut self, events: VecDeque<std::sync::Arc<dyn TEvent>>) {
		(self as &mut T).set_events(events)
	}
}

impl<T: TRepository> TRepository for std::sync::Arc<tokio::sync::RwLock<T>> {
	fn set_events(&mut self, events: VecDeque<std::sync::Arc<dyn TEvent>>) {
		tokio::task::block_in_place(move || {
			Handle::current().block_on(async move { self.write().await.set_events(events) });
		});
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
