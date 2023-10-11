use crate::prelude::Message;
use std::any::Any;

pub type AnyError = dyn Any + Send + Sync;

#[derive(Debug)]
pub enum BaseError {
	NotFound,
	StopSentinel,
	TransactionError,
	StopSentinelWithEvent(Box<dyn Message>),
	DatabaseError(Box<AnyError>),
	ServiceError(Box<AnyError>),
}

pub trait ApplicationResponse: 'static {}

pub trait ApplicationError: 'static + std::fmt::Debug {}
impl ApplicationError for BaseError {}

impl From<BaseError> for Box<dyn ApplicationError> {
	fn from(value: BaseError) -> Self {
		Box::new(value)
	}
}
