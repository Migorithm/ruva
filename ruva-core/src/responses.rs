use crate::prelude::TEvent;

#[derive(Debug, Clone)]
pub enum BaseError {
	NotFound,
	StopSentinel,
	TransactionError,
	StopSentinelWithEvent(std::sync::Arc<dyn TEvent>),
	DatabaseError(String),
	ServiceError,
}

pub trait ApplicationResponse: Send + Sync {}

pub trait ApplicationError: 'static + std::fmt::Debug + Send + Sync {}
impl ApplicationError for BaseError {}

impl From<BaseError> for Box<dyn ApplicationError> {
	fn from(value: BaseError) -> Self {
		Box::new(value)
	}
}

impl ApplicationResponse for () {}

impl ApplicationError for () {}
