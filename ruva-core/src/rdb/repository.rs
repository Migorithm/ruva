use sqlx::{PgConnection, PgPool};

use crate::{
	prelude::{BaseError, TUnitOfWork},
	prepare_bulk_operation,
	repository::Context,
};

impl Context {
	pub fn transaction(&mut self) -> &mut PgConnection {
		match self.pg_transaction.as_mut() {
			Some(trx) => trx,
			None => panic!("Transaction Has Not Begun!"),
		}
	}

	pub(crate) async fn save_outbox(&mut self) -> Result<(), BaseError> {
		let outboxes = self.events.iter().filter(|e| e.externally_notifiable()).map(|o| o.outbox()).collect::<Vec<_>>();

		prepare_bulk_operation!(
			&outboxes,
			id: i64,
			aggregate_id: String,
			aggregate_name:String,
			topic: String,
			state: String
		);
		sqlx::query(
			r#"
            INSERT INTO service_outbox
                (id, aggregate_id, topic, state, aggregate_name)
            SELECT * FROM UNNEST
                ($1::BIGINT[], $2::text[],  $3::text[], $4::text[], $5::text[])
            "#,
		)
		.bind(&id)
		.bind(&aggregate_id)
		.bind(&topic)
		.bind(&state)
		.bind(&aggregate_name)
		.execute(self.transaction())
		.await
		.map_err(|err| {
			tracing::error!("failed to insert outbox! {}", err);
			BaseError::DatabaseError(err.to_string())
		})?;
		Ok(())
	}
}

impl TUnitOfWork for Context {
	async fn begin(&mut self) -> Result<(), BaseError> {
		match self.pg_transaction.as_mut() {
			None => {
				let trx = &self.inner.write().await.conn;

				if let Some(trx) = trx.downcast_ref::<&PgPool>().or(trx.downcast_ref::<PgPool>().as_ref()) {
					self.pg_transaction = Some(trx.begin().await?);
				} else {
					tracing::error!("Transaction Error!");
					return Err(BaseError::TransactionError);
				}
				// simplify above

				Ok(())
			}
			Some(_trx) => {
				tracing::warn!("Transaction Begun Already!");
				Err(BaseError::TransactionError)?
			}
		}
	}

	async fn _commit(&mut self) -> Result<(), BaseError> {
		match self.pg_transaction.take() {
			None => panic!("Tranasction Has Not Begun!"),
			Some(trx) => Ok(trx.commit().await?),
		}
	}

	async fn rollback(&mut self) -> Result<(), BaseError> {
		self.events.clear();
		match self.pg_transaction.take() {
			None => panic!("Tranasction Has Not Begun!"),
			Some(trx) => Ok(trx.rollback().await?),
		}
	}
	async fn close(&mut self) {
		match self.pg_transaction.take() {
			None => (),
			Some(trx) => {
				let _ = trx.rollback().await;
			}
		}
	}

	async fn process_internal_events(&mut self) -> Result<(), BaseError> {
		self.send_internally_notifiable_messages().await;
		Ok(())
	}

	async fn process_external_events(&mut self) -> Result<(), BaseError> {
		self.save_outbox().await?;
		Ok(())
	}
}
