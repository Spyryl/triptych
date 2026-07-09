use std::env::VarError;
use std::error::Error as StdError;
use std::fmt::{self, Display};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use thiserror::Error;

use crate::core::diagnostic::{DiagnosticEnvelope, ErrorSeverity};
use crate::core::time::now_utc;

/// Convenience Result type using CoreError as the default error.
pub type Result<T, E = CoreError> = std::result::Result<T, E>;

const DEFAULT_CODE: &str = "ERR_GENERIC";
const DEFAULT_SEVERITY: &str = "error";
const DEFAULT_SOURCE: &str = "unknown";

/// One breadcrumb in the path that led to the current error boundary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorFrame {
    pub source: String,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

/// Structured error payload used by project code.
///
/// This is the Rust equivalent of the Node `ManagedError`: operational code
/// should raise this shape, then outer layers can enrich it with more context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManagedError {
    pub message: String,
    pub code: String,
    pub severity: String,
    pub source: String,
    #[serde(rename = "statusCode", skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,
    pub timestamp: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub path: Vec<ErrorFrame>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<Box<ErrorFrame>>,
}

impl ManagedError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: DEFAULT_CODE.to_string(),
            severity: DEFAULT_SEVERITY.to_string(),
            source: DEFAULT_SOURCE.to_string(),
            status_code: None,
            details: None,
            context: None,
            timestamp: now_utc(),
            path: Vec::new(),
            cause: None,
        }
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = code.into();
        self
    }

    pub fn with_severity(mut self, severity: impl Into<String>) -> Self {
        self.severity = severity.into();
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    pub fn with_status_code(mut self, status_code: u16) -> Self {
        self.status_code = Some(status_code);
        self
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_context(mut self, context: Value) -> Self {
        self.context = Some(context);
        self
    }

    pub fn with_cause(mut self, cause: ErrorFrame) -> Self {
        self.cause = Some(Box::new(cause));
        self
    }

    pub fn frame(&self) -> ErrorFrame {
        ErrorFrame {
            source: self.source.clone(),
            code: self.code.clone(),
            message: self.message.clone(),
            details: self.details.clone(),
        }
    }

    pub fn enrich(
        mut self,
        source: impl Into<String>,
        code: impl Into<String>,
        message: Option<String>,
        status_code: Option<u16>,
        details: Option<Value>,
        context: Option<Value>,
    ) -> Self {
        self.path.push(self.frame());
        self.source = source.into();
        self.code = code.into();
        if let Some(message) = message {
            self.message = message;
        }
        if let Some(status_code) = status_code {
            self.status_code = Some(status_code);
        }
        self.details = merge_json_object(self.details.take(), details);
        self.context = merge_json_object(self.context.take(), context);
        self.timestamp = now_utc();
        self
    }

    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or_else(|_| {
            json!({
                "message": self.message,
                "code": self.code,
                "severity": self.severity,
                "source": self.source,
                "statusCode": self.status_code,
            })
        })
    }

    pub fn to_diagnostic(&self) -> DiagnosticEnvelope {
        let severity = match self.severity.as_str() {
            "critical" => ErrorSeverity::Critical,
            "fatal" => ErrorSeverity::Fatal,
            _ => ErrorSeverity::Error,
        };

        let mut envelope = DiagnosticEnvelope::error_with_severity(
            severity,
            &self.code,
            &self.message,
            &self.source,
        );
        envelope.status_code = self.status_code;
        envelope.details = self.details.clone();
        envelope.context = self.context.clone();
        envelope.created_at = self.timestamp;
        envelope
    }
}

impl Display for ManagedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{} @ {}]", self.message, self.code, self.source)
    }
}

impl StdError for ManagedError {}

/// Core error type for the repairs library.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("{0}")]
    Managed(ManagedError),

    #[error("{0}")]
    External(ManagedError),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("environment variable error: {0}")]
    Env(#[from] VarError),

    #[error("record not found")]
    NotFound,
}

impl CoreError {
    pub fn create(error: ManagedError) -> Self {
        Self::Managed(error)
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Managed(
            ManagedError::new(message)
                .with_code("VALIDATION_FAILED")
                .with_status_code(400),
        )
    }

    pub fn validation_at(
        source: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Managed(
            ManagedError::new(message)
                .with_source(source)
                .with_code(code)
                .with_status_code(400),
        )
    }

    pub fn validation_with_details(
        source: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
        details: Value,
    ) -> Self {
        Self::Managed(
            ManagedError::new(message)
                .with_source(source)
                .with_code(code)
                .with_status_code(400)
                .with_details(details),
        )
    }

    pub fn invalid_id(message: impl Into<String>) -> Self {
        Self::Managed(
            ManagedError::new(message)
                .with_code("INVALID_ID")
                .with_status_code(400),
        )
    }

    pub fn invalid_id_at(
        source: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
        details: Value,
    ) -> Self {
        Self::Managed(
            ManagedError::new(message)
                .with_source(source)
                .with_code(code)
                .with_status_code(400)
                .with_details(details),
        )
    }

    pub fn config(message: impl Into<String>) -> Self {
        Self::Managed(
            ManagedError::new(message)
                .with_code("CONFIG_ERROR")
                .with_status_code(500),
        )
    }

    pub fn custom(message: impl Into<String>) -> Self {
        Self::Managed(
            ManagedError::new(message)
                .with_code("ERR_CUSTOM")
                .with_status_code(500),
        )
    }

    pub fn custom_at(
        source: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
        status_code: u16,
        details: Option<Value>,
    ) -> Self {
        let mut managed = ManagedError::new(message)
            .with_source(source)
            .with_code(code)
            .with_status_code(status_code);
        if let Some(details) = details {
            managed = managed.with_details(details);
        }
        Self::Managed(managed)
    }

    pub fn external_at(
        source: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
        status_code: u16,
        details: Option<Value>,
    ) -> Self {
        let mut managed = ManagedError::new(message)
            .with_source(source)
            .with_code(code)
            .with_status_code(status_code);
        if let Some(details) = details {
            managed = managed.with_details(details);
        }
        Self::External(managed)
    }

    pub fn enrich(
        self,
        source: impl Into<String>,
        code: impl Into<String>,
        message: Option<String>,
        status_code: Option<u16>,
        details: Option<Value>,
        context: Option<Value>,
    ) -> Self {
        match self {
            Self::Managed(error) => {
                Self::Managed(error.enrich(source, code, message, status_code, details, context))
            }
            other => {
                let cause = other.error_frame();
                let managed = ManagedError::new(message.unwrap_or_else(|| cause.message.clone()))
                    .with_source(source)
                    .with_code(code)
                    .with_status_code(status_code.unwrap_or(500))
                    .with_cause(cause);
                let managed = if let Some(details) = details {
                    managed.with_details(details)
                } else {
                    managed
                };
                let managed = if let Some(context) = context {
                    managed.with_context(context)
                } else {
                    managed
                };
                Self::Managed(managed)
            }
        }
    }

    pub fn status_code(&self) -> u16 {
        match self {
            Self::Managed(error) => error.status_code.unwrap_or(500),
            Self::NotFound => 404,
            Self::External(error) => error.status_code.unwrap_or(500),
            Self::Json(_) | Self::Env(_) => 500,
        }
    }

    pub fn code(&self) -> &str {
        match self {
            Self::Managed(error) => &error.code,
            Self::External(error) => &error.code,
            Self::NotFound => "NOT_FOUND",
            Self::Json(_) => "JSON_ERROR",
            Self::Env(_) => "ENV_ERROR",
        }
    }

    pub fn to_managed(&self) -> ManagedError {
        match self {
            Self::Managed(error) | Self::External(error) => error.clone(),
            other => {
                let frame = other.error_frame();
                ManagedError::new(frame.message)
                    .with_source(frame.source)
                    .with_code(frame.code)
                    .with_status_code(other.status_code())
            }
        }
    }

    pub fn to_json(&self) -> Value {
        self.to_managed().to_json()
    }

    pub fn to_diagnostic(&self) -> DiagnosticEnvelope {
        self.to_managed().to_diagnostic()
    }

    fn error_frame(&self) -> ErrorFrame {
        match self {
            Self::Managed(error) | Self::External(error) => error.frame(),
            Self::NotFound => ErrorFrame {
                source: "core".to_string(),
                code: "NOT_FOUND".to_string(),
                message: "record not found".to_string(),
                details: None,
            },
            Self::Json(error) => ErrorFrame {
                source: "serde_json".to_string(),
                code: "JSON_ERROR".to_string(),
                message: error.to_string(),
                details: None,
            },
            Self::Env(error) => ErrorFrame {
                source: "env".to_string(),
                code: "ENV_ERROR".to_string(),
                message: error.to_string(),
                details: None,
            },
        }
    }
}

fn merge_json_object(base: Option<Value>, extra: Option<Value>) -> Option<Value> {
    match (base, extra) {
        (None, None) => None,
        (Some(value), None) | (None, Some(value)) => Some(value),
        (Some(Value::Object(mut base)), Some(Value::Object(extra))) => {
            merge_object(&mut base, extra);
            Some(Value::Object(base))
        }
        (Some(base), Some(extra)) => Some(json!({
            "base": base,
            "extra": extra,
        })),
    }
}

fn merge_object(base: &mut Map<String, Value>, extra: Map<String, Value>) {
    for (key, value) in extra {
        base.insert(key, value);
    }
}
