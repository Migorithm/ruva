use crate::prelude::{BaseError, TUnitOfWork};

use sqlx::{postgres::PgPool, PgConnection, Postgres, Transaction};

#[derive(Debug)]
pub struct SQLExecutor {
	pool: &'static PgPool,
	transaction: Option<Transaction<'static, Postgres>>,
}

impl SQLExecutor {
	pub fn new(pool: &'static PgPool) -> Self {
		Self { pool, transaction: None }
	}

	pub fn transaction(&mut self) -> &mut PgConnection {
		match self.transaction.as_mut() {
			Some(trx) => trx,
			None => panic!("Transaction Has Not Begun!"),
		}
	}

	pub fn in_transaction(&self) -> bool {
		self.transaction.is_some()
	}
}

impl TUnitOfWork for SQLExecutor {
	async fn begin(&mut self) -> Result<(), BaseError> {
		match self.transaction.as_mut() {
			None => {
				self.transaction = Some(self.pool.begin().await?);
				Ok(())
			}
			Some(_trx) => {
				println!("Transaction Begun Already!");
				Err(BaseError::TransactionError)?
			}
		}
	}

	async fn commit(&mut self) -> Result<(), BaseError> {
		match self.transaction.take() {
			None => panic!("Tranasction Has Not Begun!"),
			Some(trx) => Ok(trx.commit().await?),
		}
	}
	async fn rollback(&mut self) -> Result<(), BaseError> {
		match self.transaction.take() {
			None => panic!("Tranasction Has Not Begun!"),
			Some(trx) => Ok(trx.rollback().await?),
		}
	}

	async fn close(&mut self) {
		match self.transaction.take() {
			None => (),
			Some(trx) => {
				let _ = trx.rollback().await;
			}
		}
	}
}
