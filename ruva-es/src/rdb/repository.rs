use std::{marker::PhantomData, sync::Arc};

use ruva_core::{
	prelude::{
		from_value, json,
		sqlx::{self, Row},
		tokio::sync::RwLock,
		Value,
	},
	prepare_bulk_operation,
	rdb::executor::SQLExecutor,
};

use crate::{
	aggregate::{TAggregateES, TAggregateMetadata},
	event::{EventEnvolope, TEvent},
	event_store::TEventStore,
};

pub struct SqlRepository<A: TAggregateES> {
	pub executor: Arc<RwLock<SQLExecutor>>,
	_phantom: PhantomData<A>,
}

impl<A: TAggregateES + TAggregateMetadata> SqlRepository<A> {
	pub fn new(executor: Arc<RwLock<SQLExecutor>>) -> Self {
		Self {
			executor,
			_phantom: Default::default(),
		}
	}
	fn wrap_events(aggregate: &A) -> Vec<EventEnvolope> {
		let mut current_sequence = aggregate.sequence();
		let aggregate_type = aggregate.aggregate_type();
		let aggregate_id = aggregate.aggregate_id();
		aggregate
			.events()
			.iter()
			.map(|event| {
				current_sequence += 1;
				EventEnvolope {
					aggregate_type: aggregate_type.clone(),
					aggregate_id: aggregate_id.clone(),
					sequence: current_sequence,
					event_type: event.event_type(),
					event_version: event.event_version(),
					payload: json!(event),
				}
			})
			.collect()
	}
}

impl<A: TAggregateES + TAggregateMetadata> TEventStore<A> for SqlRepository<A> {
	async fn load_events(&self, aggregate_id: &str) -> Result<Vec<EventEnvolope>, A::Error> {
		Ok(sqlx::query(
			r#"
            SELECT
                aggregate_type ,
                aggregate_id   ,
                sequence       ,
                event_type     ,
                event_version  ,
                payload 
            FROM events
            WHERE aggregate_id = $1
            "#,
		)
		.bind(aggregate_id)
		.map(|record| EventEnvolope {
			aggregate_type: record.get("aggregate_type"),
			aggregate_id: record.get("aggregate_id"),
			sequence: record.get("sequence"),
			event_type: record.get("event_type"),
			event_version: record.get("event_version"),
			payload: record.get("payload"),
		})
		.fetch_all(self.executor.read().await.connection())
		.await
		.unwrap())
	}

	async fn load_aggregate(&self, aggregate_id: &str) -> Result<A, A::Error> {
		let events = self.load_events(aggregate_id).await?;
		let mut aggregate = A::default();
		let current_sequence = events.len() as i64;
		events.into_iter().for_each(|event| aggregate.apply(from_value(event.payload).unwrap()));
		aggregate.set_sequence(current_sequence);
		Ok(aggregate)
	}

	async fn commit(&self, aggregate: &A) -> Result<(), A::Error> {
		let events = Self::wrap_events(aggregate);
		if events.is_empty() {
			return Ok(());
		}
		prepare_bulk_operation!(
			&events,
			aggregate_type: String,
			aggregate_id: String,
			sequence:i64,
			event_type: String,
			event_version: String,
			payload: Value
		);
		sqlx::query(
			r#"
            INSERT INTO events (
                aggregate_type ,
                aggregate_id   ,
                sequence       ,
                event_type     ,
                event_version  ,
                payload        
            )
            VALUES (
                UNNEST($1::text[]),
                UNNEST($2::text[]),
                UNNEST($3::bigint[]),
                UNNEST($4::text[]),
                UNNEST($5::text[]),
                UNNEST($6::jsonb[])
            )
            "#,
		)
		.bind(&aggregate_type)
		.bind(&aggregate_id)
		.bind(&sequence)
		.bind(&event_type)
		.bind(&event_version)
		.bind(&payload)
		.execute(self.executor.read().await.connection())
		.await
		.unwrap();
		Ok(())
	}
}
