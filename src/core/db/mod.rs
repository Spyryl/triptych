pub mod managed_record;
pub mod managed_table;
pub mod managed_table_locking;
pub mod pool;

use crate::core::error::CoreError;

impl From<tokio_postgres::Error> for CoreError {
    fn from(error: tokio_postgres::Error) -> Self {
        CoreError::external_at("postgres", "POSTGRES_ERROR", error.to_string(), 500, None)
    }
}

impl From<deadpool_postgres::PoolError> for CoreError {
    fn from(error: deadpool_postgres::PoolError) -> Self {
        CoreError::external_at("postgres.pool", "POOL_ERROR", error.to_string(), 500, None)
    }
}
