use std::{error, fmt::Display};

use crate::prelude::Message;

pub type AnyError = dyn error::Error + Send + Sync;

#[derive(Debug)]
pub enum BaseError {
	EventNotFound,
	CommandNotFound,
	StopSentinel,
	TransactionError,
	StopSentinelWithEvent(Box<dyn Message>),
	DatabaseError(Box<AnyError>),
	ServiceError(Box<AnyError>),
}

impl std::error::Error for BaseError {}
impl Display for BaseError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			BaseError::CommandNotFound => write!(f, "CommandNotFound"),
			BaseError::EventNotFound => write!(f, "EventNotFound"),
			BaseError::StopSentinel => write!(f, "StopSentinel"),
			BaseError::StopSentinelWithEvent(_message) => write!(f, "StopSentinel"),
			BaseError::DatabaseError(res) => write!(f, "{}", res),
			BaseError::ServiceError(_) => write!(f, "ServiceError"),
			BaseError::TransactionError => write!(f, "TransactionError"),
		}
	}
}

pub trait ApplicationResponse: 'static {}

pub trait ApplicationError: std::error::Error + 'static {}
impl ApplicationError for BaseError {}

impl From<BaseError> for Box<dyn ApplicationError> {
	fn from(value: BaseError) -> Self {
		Box::new(value)
	}
}
