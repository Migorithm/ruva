use std::{any::Any, collections::VecDeque, fmt::Debug};

use downcast_rs::{impl_downcast, Downcast};
use serde::Serialize;
use serde_json::Value;

use crate::outbox::OutBox;

// Aggregate!
pub trait Aggregate: Send + Sync + Default {
    fn collect_events(&mut self) -> VecDeque<Box<dyn Message>> {
        if !self.events().is_empty() {
            self.take_events()
        } else {
            VecDeque::new()
        }
    }
    fn events(&self) -> &VecDeque<Box<dyn Message>>;

    fn take_events(&mut self) -> VecDeque<Box<dyn Message>>;
    fn raise_event(&mut self, event: Box<dyn Message>);
}

pub trait Buildable<B: Aggregate> {
    fn builder() -> Builder<B>;
}

pub struct Builder<T: Aggregate>(pub T);

impl<T: Aggregate> Builder<T> {
    pub fn new() -> Self {
        Builder(T::default())
    }
    pub fn build(self) -> T {
        self.0
    }
}

impl<T: Aggregate> Default for Builder<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
macro_rules! Aggregate {
    (

        $( #[$attr:meta] )*
        $pub:vis
        struct $aggregate:ident {
            $(#[$field_attr:meta])*
            $($field_pub:vis $field_name:ident :$field_type:ty),*
        $(,)?}
    ) => {

        $( #[$attr])*


        impl $crate::domain::Aggregate for $aggregate {
            fn events(&self) -> &VecDeque<Box<dyn Message>> {
                &self.events
            }
            fn take_events(&mut self) -> VecDeque<Box<dyn Message>> {
                mem::take(&mut self.events)
            }
            fn raise_event(&mut self, event: Box<dyn Message>) {
                self.events.push_back(event)
            }
        }

        impl $crate::domain::Buildable<$aggregate> for $aggregate {
            fn builder() -> Builder<$aggregate> {
                Builder::<$aggregate>::new()
            }
        }
    };
}

#[macro_export]
macro_rules! Entity {
    (

        $( #[$attr:meta] )*
        $pub:vis
        struct $classic:ident {
            $(#[$field_attr:meta])*
            $($field_pub:vis $field_name:ident :$field_type:ty),*
    $(,)?}
) => {
        impl $classic {
            $(
                paste::paste!{
                pub fn [< set_ $field_name >] (&mut self, $field_name:$field_type){
                    self.$field_name = $field_name
                }
            }
            )*

        }
    };
}

pub trait Message: Sync + Send + Any + Downcast {
    fn externally_notifiable(&self) -> bool {
        false
    }
    fn internally_notifiable(&self) -> bool {
        false
    }

    fn metadata(&self) -> MessageMetadata;
    fn outbox(&self) -> Box<dyn OutBox>;
    // {
    //     let metadata = self.metadata();
    //     Outbox::new(metadata.aggregate_id, metadata.topic, self.state())
    // }
    fn message_clone(&self) -> Box<dyn Message>;

    fn state(&self) -> String;

    fn to_message(self) -> Box<dyn Message + 'static>;
}
impl_downcast!(Message);
impl Debug for dyn Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.metadata().topic)
    }
}

pub struct MessageMetadata {
    pub aggregate_id: String,
    pub topic: String,
}

// Trait To Mark Event As Mail Sendable. Note that template_name must be specified.
pub trait MailSendable: Message + Serialize + Send + Sync + 'static {
    fn template_name(&self) -> String;
    fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

#[macro_export]
macro_rules! message {
    ($event:ty $(, $v1:ident $(, $v2:ident)? )? ) => {
        impl $crate::domain::Message for $event {
            fn metadata(&self) -> $crate::domain::MessageMetadata {
                $crate::domain::MessageMetadata {
                    aggregate_id: self.id.to_string(),
                    topic: stringify!($event).into(),
                }
            }
            fn message_clone(&self) -> Box<dyn $crate::domain::Message> {
                Box::new(self.clone())
            }
            fn state(&self) -> String {
                serde_json::to_string(&self).expect("Failed to serialize")
            }
            fn to_message(self)-> Box<dyn $crate::domain::Message+'static>{
                Box::new(self)
            }
            fn outbox(&self) -> Box<dyn $crate::outbox::OutBox>
            {
                let metadata = self.metadata();
                Box::new(Outbox::new(metadata.aggregate_id, metadata.topic, self.state()))
            }


            $(fn $v1(&self) -> bool {
                true
            }
            $(fn $v2(&self) -> bool {
                true
            })?
        )?
        }
    };
}

pub trait Command: 'static + Send + Any + Sync {}
