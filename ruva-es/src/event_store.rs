use crate::{aggregate::TAggregateES, event::EventEnvolope};

pub trait TEventStore<A: TAggregateES>: Sync + Send {
	fn load_events(&self, aggregate_id: &str) -> impl std::future::Future<Output = Result<Vec<EventEnvolope>, A::Error>> + Send;
	fn load_aggregate(&self, aggregate_id: &str) -> impl std::future::Future<Output = Result<A, A::Error>> + Send;

	fn commit(&self, aggregate: &A) -> impl std::future::Future<Output = Result<(), A::Error>> + Send;
}
