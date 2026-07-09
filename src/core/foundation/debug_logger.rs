use std::{
    collections::HashMap,
    fmt::Debug,
    sync::Mutex,
    time::{Duration, Instant},
};

use once_cell::sync::Lazy;

use crate::core::time::now_utc_iso;

static DEBUG_ENV: Lazy<Option<String>> = Lazy::new(|| std::env::var("DEBUG").ok());

/// Simple debug logger that respects the DEBUG environment variable.
///
/// Set DEBUG=* to enable all modules, or DEBUG=module1,module2 for specific ones.
/// Set DEBUG=-module1 to enable all except module1.
#[derive(Debug)]
pub struct DebugLogger {
    module: String,
    enabled: bool,
    timers: Mutex<HashMap<String, Instant>>,
}

impl DebugLogger {
    pub fn new(module: impl Into<String>) -> Self {
        let module = module.into();
        let enabled = is_module_enabled(&module);
        Self {
            module,
            enabled,
            timers: Mutex::new(HashMap::new()),
        }
    }

    /// Alias for new() - matches the Python pattern.
    pub fn get(module: impl Into<String>) -> Self {
        Self::new(module)
    }

    pub fn log(&self, message: impl AsRef<str>) {
        if self.enabled {
            println!("{} [{}] {}", timestamp(), self.module, message.as_ref());
        }
    }

    pub fn info(&self, message: impl AsRef<str>) {
        if self.enabled {
            println!(
                "{} [{}] [INFO] {}",
                timestamp(),
                self.module,
                message.as_ref()
            );
        }
    }

    pub fn warn(&self, message: impl AsRef<str>) {
        // Warnings always print regardless of enabled state
        println!(
            "{} [{}] [WARN] {}",
            timestamp(),
            self.module,
            message.as_ref()
        );
    }

    pub fn error(&self, message: impl AsRef<str>) {
        // Errors always print to stderr
        eprintln!(
            "{} [{}] [ERROR] {}",
            timestamp(),
            self.module,
            message.as_ref()
        );
    }

    pub fn value<V>(&self, label: impl AsRef<str>, value: V)
    where
        V: Debug,
    {
        if self.enabled {
            println!(
                "{} [{}] {}: {:?}",
                timestamp(),
                self.module,
                label.as_ref(),
                value
            );
        }
    }

    /// Start a timer with the given label.
    pub fn time(&self, label: impl Into<String>) {
        if !self.enabled {
            return;
        }
        let mut timers = self
            .timers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        timers.insert(label.into(), Instant::now());
    }

    /// End a timer and log the elapsed time.
    pub fn time_end(&self, label: impl AsRef<str>) {
        if !self.enabled {
            return;
        }
        let mut timers = self
            .timers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(start) = timers.remove(label.as_ref()) {
            let elapsed = Instant::now().saturating_duration_since(start);
            println!(
                "{} [{}] {} completed in {} ms",
                timestamp(),
                self.module,
                label.as_ref(),
                as_millis(elapsed)
            );
        }
    }
}

fn timestamp() -> String {
    now_utc_iso()
}

fn as_millis(duration: Duration) -> u128 {
    duration.as_millis()
}

fn is_module_enabled(module: &str) -> bool {
    match &*DEBUG_ENV {
        None => false,
        Some(value) if value.trim().is_empty() => false,
        Some(value) if value == "*" => true,
        Some(value) if value.starts_with('-') => {
            // Exclude mode: enable all except listed modules
            !value
                .trim_start_matches('-')
                .split(',')
                .map(str::trim)
                .any(|entry| !entry.is_empty() && entry == module)
        }
        Some(value) => {
            // Include mode: only enable listed modules
            value
                .split(',')
                .map(str::trim)
                .any(|entry| !entry.is_empty() && entry == module)
        }
    }
}
