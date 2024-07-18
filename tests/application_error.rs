use ruva::*;

#[test]
fn application_error_derive_test() {
	#[derive(Debug, ApplicationError)]
	#[crates(ruva)]
	enum Err {
		#[stop_sentinel]
		Items,
		#[stop_sentinel_with_event]
		StopSentinelWithEvent(std::sync::Arc<dyn TEvent>),
		#[database_error]
		DatabaseError(String),
		BaseError(BaseError),
	}

	impl std::fmt::Display for Err {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			match self {
				Self::Items => write!(f, "items"),
				Self::StopSentinelWithEvent(item) => write!(f, "{:?}", item),
				Self::DatabaseError(err) => write!(f, "{:?}", err),
				Self::BaseError(err) => write!(f, "{:?}", err),
			}
		}
	}
}
