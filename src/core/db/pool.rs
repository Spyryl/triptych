use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;

use crate::core::error::{CoreError, Result};
use crate::core::foundation::config::DatabaseConfig;

pub fn create_db_pool(config: &DatabaseConfig) -> Result<Pool> {
    let pg_config = config
        .connection_string()
        .parse::<tokio_postgres::Config>()
        .map_err(|error| CoreError::config(format!("invalid database config: {}", error)))?;
    let manager_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };
    let manager = Manager::from_config(pg_config, NoTls, manager_config);

    Pool::builder(manager)
        .max_size(config.max_pool_size)
        .runtime(Runtime::Tokio1)
        .build()
        .map_err(|error| CoreError::config(format!("failed to build database pool: {}", error)))
}
