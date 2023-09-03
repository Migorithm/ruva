use async_trait::async_trait;
use downcast_rs::{impl_downcast, Downcast};

use serde::Serialize;
use serde_json::Value;
use std::{any::Any, collections::VecDeque, fmt::Debug};

use chrono::{DateTime, Utc};

use uuid::Uuid;

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
macro_rules! MailSendableMacro {
    (

        $( #[$attr:meta] )*
        $pub:vis
        struct $mail_sendable:ident {
            $(
                $(#[$field_attr:meta])*
                $field_vis:vis // this visibility will be applied to the getters instead
                $field_name:ident : $field_type:ty
            ),* $(,)?
    }
    ) => {
        $( #[$attr])*
        impl $crate::lib_components::MailSendable for $mail_sendable {
            fn template_name(&self) -> String {
                // * subject to change
                stringify!($mail_sendable).into()
            }
        }
    };
}

pub trait Command: 'static + Send + Any + Sync {}

pub trait Aggregate: Send + Sync + Default {
	fn collect_events(&mut self) -> VecDeque<Box<dyn Message>> {
		if !self.events().is_empty() {
			self.take_events()
		} else {
			VecDeque::new()
		}
	}
	fn events(&self) -> &std::collections::VecDeque<Box<dyn Message>>;

	fn take_events(&mut self) -> std::collections::VecDeque<Box<dyn Message>>;
	fn raise_event(&mut self, event: Box<dyn Message>);
}

#[macro_export]
macro_rules! Entity {
    (

        $( #[$attr:meta] )*
        $pub:vis
        struct $classic:ident {
            $(
                $(#[$field_attr:meta])*
                $field_vis:vis // this visibility will be applied to the getters instead
                $field_name:ident : $field_type:ty
            ),* $(,)?
    }
) => {
        impl $classic {
            $(
                $crate::paste!{
                pub fn [< set_ $field_name >] (mut self, $field_name:$field_type)-> Self{
                    self.$field_name = $field_name;
                    self

                }
            }
            )*

        }
    };
}

#[async_trait]
pub trait OutBox: Send + Sync + 'static {
	fn convert_event(&self) -> Box<dyn Message>;
	fn tag_processed(&mut self);

	fn id(&self) -> Uuid;
	fn aggregate_id(&self) -> &str;
	fn topic(&self) -> &str;
	fn state(&self) -> &str;
	fn processed(&self) -> bool;
	fn create_dt(&self) -> DateTime<Utc>;
}

#[macro_export]
macro_rules! convert_event {
    ( $obj:expr, $( $type: ty ), * ) => {
        match $obj.topic.as_str() {
          $(stringify!($type)=> serde_json::from_str::<$type>($obj.state.as_str()).expect("Given type not deserializable!").message_clone() ,)*
          _ => {
                panic!("Such event not allowed to process through outbox.");
          }
        }
    };
}
