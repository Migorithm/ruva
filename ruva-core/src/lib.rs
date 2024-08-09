mod aggregate;
mod backtrace;
mod bus_components;
mod message;
mod outbox;
mod rdb;
mod repository;
mod responses;
mod snowflake;
mod unit_of_work;

pub mod prelude {
	pub use crate::aggregate::*;
	pub use crate::bus_components::contexts::AtomicContextManager;
	pub use crate::bus_components::contexts::ContextManager;

	pub use crate::bus_components::handler::*;
	pub use crate::bus_components::messagebus::*;
	pub use crate::message::*;
	pub use crate::outbox::OutBox;
	#[cfg(feature = "sqlx-postgres")]
	pub use crate::rdb::repository::SqlRepository;
	pub use crate::repository::TRepository;

	pub use crate::responses::{ApplicationError, ApplicationResponse, BaseError};
	pub use crate::snowflake::SnowFlake;
	pub use crate::unit_of_work::*;

	pub use async_trait::async_trait;
	pub use hashbrown::HashMap as HandlerMapper;
	pub use serde;
	pub use serde::{Deserialize, Serialize};
	pub use serde_json;
	#[cfg(feature = "sqlx-postgres")]
	pub use sqlx;
	pub use tokio;
	pub use tracing;
}

pub mod event_macros {
	// pub use crate::init_command_handler;
	// pub use crate::init_event_handler;
	pub use crate::prepare_bulk_operation;
}
