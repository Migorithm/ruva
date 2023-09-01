use std::{error, fmt::Display};

use downcast_rs::{impl_downcast, Downcast};

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

pub trait ApplicationError: std::error::Error + Downcast + 'static {}
impl ApplicationError for BaseError {}

impl From<BaseError> for Box<dyn ApplicationError> {
	fn from(value: BaseError) -> Self {
		Box::new(value)
	}
}

impl_downcast!(ApplicationError);

#[macro_export]
macro_rules! ApplicationError {
	(
        $( #[$attr:meta] )*
        $pub:vis
        enum $error_enum:ident {
            $(#[$field_attr:meta])*
            $($variant:ident$($value:ty)?),*$(,)?
        }
    ) => {
		impl std::error::Error for $error_enum {}
		impl $crate::responses::ApplicationError for $error_enum {}
		impl From<$crate::responses::BaseError> for $error_enum {
			fn from(value: $crate::responses::BaseError) -> Self {
				$error_enum::BaseError(value)
			}
		}
		impl From<std::boxed::Box<$error_enum>> for std::boxed::Box<dyn $crate::responses::ApplicationError> {
			fn from(value: std::boxed::Box<$error_enum>) -> Self {
				value
			}
		}
		impl From<$error_enum> for std::boxed::Box<dyn $crate::responses::ApplicationError> {
			fn from(value: $error_enum) -> Self {
				std::boxed::Box::new(value)
			}
		}
	};
}
