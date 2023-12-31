use crate::prelude::{BaseError, TUnitOfWork};

use sqlx::{pool::PoolOptions, postgres::PgConnectOptions, postgres::PgPool, ConnectOptions, Postgres, Transaction};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct SQLExecutor {
	pool: &'static PgPool,
	transaction: Option<Transaction<'static, Postgres>>,
}

impl SQLExecutor {
	pub fn new() -> Arc<RwLock<Self>> {
		Arc::new(RwLock::new(Self {
			pool: connection_pool(),
			transaction: None,
		}))
	}
	pub fn transaction(&mut self) -> &mut Transaction<'static, Postgres> {
		match self.transaction.as_mut() {
			Some(trx) => trx,
			None => panic!("Transaction Has Not Begun!"),
		}
	}
	pub fn connection(&self) -> &PgPool {
		self.pool
	}
}

impl TUnitOfWork for SQLExecutor {
	async fn begin(&mut self) -> Result<(), BaseError> {
		match self.transaction.as_mut() {
			None => {
				self.transaction = Some(self.pool.begin().await.map_err(|err| BaseError::DatabaseError(Box::new(err)))?);
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
			Some(trx) => trx.commit().await.map_err(|err| BaseError::DatabaseError(Box::new(err))),
		}
	}
	async fn rollback(&mut self) -> Result<(), BaseError> {
		match self.transaction.take() {
			None => panic!("Tranasction Has Not Begun!"),
			Some(trx) => trx.rollback().await.map_err(|err| BaseError::DatabaseError(Box::new(err))),
		}
	}
}

pub fn connection_pool() -> &'static PgPool {
	static POOL: std::sync::OnceLock<PgPool> = std::sync::OnceLock::new();
	POOL.get_or_init(|| {
		std::thread::spawn(|| {
			#[tokio::main]
			async fn get_connection_pool() -> PgPool {
				let url = &std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
				let mut opts: PgConnectOptions = url.parse().unwrap();
				opts.disable_statement_logging();
				PoolOptions::new()
					.max_connections(30)
					.connect_with(opts)
					.await
					.map_err(|err| BaseError::DatabaseError(Box::new(err)))
					.unwrap()
			}
			get_connection_pool()
		})
		.join()
		.unwrap()
	})
}
