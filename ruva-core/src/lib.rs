pub use paste::paste;
pub mod aggregate;
pub mod handler;
pub mod message;
pub mod messagebus;
pub mod outbox;

pub mod rdb;
pub mod repository;
pub mod responses;
pub mod snowflake;
pub mod unit_of_work;
pub mod utils;
pub mod prelude {
	pub use crate::aggregate::*;
	pub use crate::handler::*;
	pub use crate::message::*;
	pub use crate::messagebus::*;
	pub use crate::outbox::OutBox;
	#[cfg(feature = "sqlx-postgres")]
	pub use crate::rdb;
	pub use crate::repository::TRepository;
	pub use crate::responses::*;
	pub use crate::unit_of_work::*;
	pub use crate::utils::*;
	pub use async_trait::async_trait;
	pub use hashbrown::HashMap as HandlerMapper;
	pub use paste::paste;

	pub use serde::{self, de::DeserializeOwned, Deserialize, Serialize};
	pub use serde_json::{from_value, json, Value};
	#[cfg(feature = "sqlx-postgres")]
	pub use sqlx;
	pub use tokio;
}

pub mod event_macros {
	// pub use crate::init_command_handler;
	// pub use crate::init_event_handler;
	pub use crate::prepare_bulk_operation;
}
