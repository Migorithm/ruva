use crate::prelude::BaseError;
use crate::snowflake::SnowFlake;

use sqlx::error::BoxDynError;
use sqlx::postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef};
use sqlx::{Encode, Postgres, Type};

impl Encode<'_, Postgres> for SnowFlake {
	fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> sqlx::encode::IsNull {
		let value = self.0;
		<i64 as Encode<Postgres>>::encode(value, buf)
	}
}

impl<'r> sqlx::Decode<'r, Postgres> for SnowFlake {
	fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
		let i64_value = <i64 as sqlx::Decode<Postgres>>::decode(value)?;
		Ok(SnowFlake(i64_value))
	}
}

impl sqlx::Type<Postgres> for SnowFlake {
	fn type_info() -> sqlx::postgres::PgTypeInfo {
		<i64 as Type<Postgres>>::type_info()
	}

	fn compatible(ty: &PgTypeInfo) -> bool {
		<i64 as Type<Postgres>>::compatible(ty)
	}
}

impl PgHasArrayType for SnowFlake {
	fn array_type_info() -> sqlx::postgres::PgTypeInfo {
		<i64 as PgHasArrayType>::array_type_info()
	}

	fn array_compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
		<i64 as PgHasArrayType>::array_compatible(ty)
	}
}

impl From<sqlx::Error> for BaseError {
	fn from(value: sqlx::Error) -> Self {
		eprintln!("{:?}", value);
		Self::DatabaseError(value.to_string())
	}
}
