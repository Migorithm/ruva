use std::{error, fmt::Display};

pub type AnyError = dyn error::Error + Send + Sync;

#[derive(Debug)]
pub enum BaseError {
	EventNotFound,
	CommandNotFound,
	StopSentinel,
	DatabaseConnectionError(Box<AnyError>),
	TransactionError,
}

impl std::error::Error for BaseError {}
impl Display for BaseError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			BaseError::CommandNotFound => write!(f, "CommandNotFound"),
			BaseError::EventNotFound => write!(f, "EventNotFound"),
			BaseError::StopSentinel => write!(f, "StopSentinel"),
			BaseError::TransactionError => write!(f, "TransactionError"),
			BaseError::DatabaseConnectionError(res) => write!(f, "{}", res),
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
