use async_trait::async_trait;

use chrono::{DateTime, Utc};

use uuid::Uuid;

use crate::domain::Message;

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
