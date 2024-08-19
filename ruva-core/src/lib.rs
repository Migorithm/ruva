mod adapters;
mod aggregate;
mod backtrace;
mod bus_components;
mod macros;
mod message;
mod outbox;
mod responses;
mod snowflake;
mod unit_of_work;

pub mod prelude {
	pub use crate::aggregate::*;
	pub use crate::bus_components::contexts::AtomicContextManager;
	pub use crate::bus_components::contexts::Context;
	pub use crate::bus_components::contexts::ContextManager;
	pub use crate::bus_components::contexts::TSetCurrentEvents;
	pub use crate::bus_components::handler::*;
	pub use crate::bus_components::messagebus::*;
	pub use crate::message::*;
	pub use crate::outbox::OutBox;
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
	pub use crate::error;
	pub use crate::make_conversion;
	pub use crate::make_smart_pointer;
	pub use crate::prepare_bulk_operation;
}
