use std::env;

use once_cell::sync::Lazy;

/// PostgreSQL database configuration.
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password: String,
    pub max_pool_size: usize,
    pub idle_timeout_millis: u64,
    pub connection_timeout_millis: u64,
}

/// Debug/logging configuration.
#[derive(Debug, Clone)]
pub struct DebugConfig {
    pub enabled_modules: Option<String>,
    pub log_file: Option<String>,
    pub level: String,
}

/// Performance tuning configuration.
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    pub cache_enabled: bool,
    pub cache_ttl_secs: u64,
    pub batch_size: u32,
    pub query_timeout_millis: u64,
}

/// Application runtime configuration.
#[derive(Debug, Clone)]
pub struct AppRuntime {
    pub port: u16,
    pub env: String,
    pub timezone: String,
}

/// Main application configuration, loaded from environment variables.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub db: DatabaseConfig,
    pub debug: DebugConfig,
    pub performance: PerformanceConfig,
    pub runtime: AppRuntime,
}

impl AppConfig {
    /// Load configuration from environment variables.
    pub fn from_env() -> Self {
        Self {
            db: DatabaseConfig::from_env(),
            debug: DebugConfig::from_env(),
            performance: PerformanceConfig::from_env(),
            runtime: AppRuntime::from_env(),
        }
    }

    /// Get the global singleton configuration instance.
    pub fn global() -> &'static AppConfig {
        static INSTANCE: Lazy<AppConfig> = Lazy::new(AppConfig::from_env);
        &INSTANCE
    }
}

impl DatabaseConfig {
    fn from_env() -> Self {
        Self {
            host: env::var("MANDATE_DB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env_var("MANDATE_DB_PORT", 5432),
            database: env::var("MANDATE_DB_NAME").unwrap_or_else(|_| "mandate".to_string()),
            user: env::var("MANDATE_DB_USER").unwrap_or_else(|_| "mandate_api".to_string()),
            password: env::var("MANDATE_DB_PASSWORD").unwrap_or_default(),
            max_pool_size: env_var("MANDATE_DB_MAX_POOL", 20),
            idle_timeout_millis: env_var("MANDATE_DB_IDLE_TIMEOUT", 30_000),
            connection_timeout_millis: env_var("MANDATE_DB_CONNECTION_TIMEOUT", 2_000),
        }
    }

    /// Build a PostgreSQL connection string from the config.
    pub fn connection_string(&self) -> String {
        format!(
            "host={} port={} dbname={} user={} password={}",
            self.host, self.port, self.database, self.user, self.password
        )
    }
}

impl DebugConfig {
    fn from_env() -> Self {
        Self {
            enabled_modules: env::var("DEBUG").ok(),
            log_file: env::var("DEBUG_FILE").ok(),
            level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        }
    }
}

impl PerformanceConfig {
    fn from_env() -> Self {
        Self {
            cache_enabled: env::var("CORE_CACHE_ENABLED")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(true),
            cache_ttl_secs: env_var("CORE_CACHE_TTL", 300),
            batch_size: env_var("CORE_BATCH_SIZE", 1000),
            query_timeout_millis: env_var("CORE_QUERY_TIMEOUT", 30_000),
        }
    }
}

impl AppRuntime {
    fn from_env() -> Self {
        Self {
            port: env_var("PORT", 3000),
            env: env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()),
            timezone: env::var("APP_TZ").unwrap_or_else(|_| "UTC".to_string()),
        }
    }

    pub fn is_production(&self) -> bool {
        self.env.eq_ignore_ascii_case("production")
    }

    pub fn is_development(&self) -> bool {
        self.env.eq_ignore_ascii_case("development")
    }
}

fn env_var<T>(key: &str, default: T) -> T
where
    T: std::str::FromStr,
{
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<T>().ok())
        .unwrap_or(default)
}
