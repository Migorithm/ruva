use crate::prelude::Message;
use std::any::Any;

pub type AnyError = dyn Any + Send + Sync;

#[derive(Debug)]
pub enum BaseError {
	NotFound,
	StopSentinel,
	TransactionError,
	StopSentinelWithEvent(std::sync::Arc<dyn Message>),
	DatabaseError(Box<AnyError>),
	ServiceError(Box<AnyError>),
}

pub trait ApplicationResponse: 'static + Send + Sync {}

pub trait ApplicationError: 'static + std::fmt::Debug + Send + Sync {}
impl ApplicationError for BaseError {}

impl From<BaseError> for Box<dyn ApplicationError> {
	fn from(value: BaseError) -> Self {
		Box::new(value)
	}
}
