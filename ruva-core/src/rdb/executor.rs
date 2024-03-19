use crate::prelude::{BaseError, TUnitOfWork};

use sqlx::{
	pool::PoolOptions,
	postgres::{PgConnectOptions, PgPool},
	ConnectOptions, PgConnection, Postgres, Transaction,
};
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct SQLExecutor {
	pool: PgPool,
	transaction: Option<Transaction<'static, Postgres>>,
}

impl SQLExecutor {
	pub fn new() -> Arc<RwLock<Self>> {
		Arc::new(RwLock::new(Self {
			pool: connection_pool(),
			transaction: None,
		}))
	}

	pub fn transaction(&mut self) -> &mut PgConnection {
		match self.transaction.as_mut() {
			Some(trx) => trx,
			None => panic!("Transaction Has Not Begun!"),
		}
	}
	pub fn connection(&self) -> &PgPool {
		&self.pool
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

pub trait TCloneExecutor<T>
where
	T: TUnitOfWork,
{
	fn clone_executor(&self) -> Arc<RwLock<T>>;
}

static INIT: OnceLock<PgPool> = OnceLock::new();

pub fn connection_pool() -> PgPool {
	INIT.get_or_init(|| {
		let url = &std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
		let opts: PgConnectOptions = url.parse::<PgConnectOptions>().unwrap().disable_statement_logging();

		let mut pool_options = PoolOptions::new().acquire_timeout(std::time::Duration::from_secs(2)).max_connections(1);
		if cfg!(test) {
			pool_options = pool_options.test_before_acquire(false)
		};
		pool_options.connect_lazy_with(opts)
	})
	.clone()
}
