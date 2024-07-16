use downcast_rs::{impl_downcast, Downcast};

pub trait TConnection: Send + Sync + Downcast {}

#[cfg(feature = "sqlx-postgres")]
impl TConnection for &'static sqlx::postgres::PgPool {}
#[cfg(feature = "sqlx-postgres")]
impl TConnection for Box<&'static sqlx::postgres::PgPool> {}

#[cfg(feature = "sqlx-postgres")]
impl TConnection for sqlx::postgres::PgPool {}
#[cfg(feature = "sqlx-postgres")]
impl TConnection for Box<sqlx::postgres::PgPool> {}

#[cfg(feature = "sqlx-postgres")]
impl TConnection for &'static mut sqlx::PgConnection {}
#[cfg(feature = "sqlx-postgres")]
impl TConnection for Box<&'static mut sqlx::PgConnection> {}

// Design TConnection so each different connection can be implemented and return itself

impl_downcast!(TConnection);
