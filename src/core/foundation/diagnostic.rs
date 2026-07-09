use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::time::now_utc;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticKind {
    Error,
    Log,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Warn,
    Info,
    Debug,
}

impl LogLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorSeverity {
    Error,
    Critical,
    Fatal,
}

impl ErrorSeverity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Critical => "critical",
            Self::Fatal => "fatal",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticLevel {
    Log(LogLevel),
    Error(ErrorSeverity),
}

impl DiagnosticLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Log(level) => level.as_str(),
            Self::Error(severity) => severity.as_str(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiagnosticEnvelope {
    pub kind: DiagnosticKind,
    pub level: DiagnosticLevel,
    pub code: String,
    pub message: String,
    pub source: String,
    #[serde(rename = "statusCode", skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_table: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<i64>,
    pub created_at: DateTime<Utc>,
}

impl DiagnosticEnvelope {
    pub fn error(
        code: impl Into<String>,
        message: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            kind: DiagnosticKind::Error,
            level: DiagnosticLevel::Error(ErrorSeverity::Error),
            code: code.into(),
            message: message.into(),
            source: source.into(),
            status_code: None,
            details: None,
            context: None,
            target_table: None,
            target_id: None,
            created_at: now_utc(),
        }
    }

    pub fn error_with_severity(
        severity: ErrorSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        let mut envelope = Self::error(code, message, source);
        envelope.level = DiagnosticLevel::Error(severity);
        envelope
    }

    pub fn log(
        level: LogLevel,
        code: impl Into<String>,
        message: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            kind: DiagnosticKind::Log,
            level: DiagnosticLevel::Log(level),
            code: code.into(),
            message: message.into(),
            source: source.into(),
            status_code: None,
            details: None,
            context: None,
            target_table: None,
            target_id: None,
            created_at: now_utc(),
        }
    }

    pub const fn error_severity(&self) -> Option<ErrorSeverity> {
        match self.level {
            DiagnosticLevel::Error(severity) => Some(severity),
            DiagnosticLevel::Log(_) => None,
        }
    }

    pub const fn log_level(&self) -> Option<LogLevel> {
        match self.level {
            DiagnosticLevel::Log(level) => Some(level),
            DiagnosticLevel::Error(_) => None,
        }
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

    pub fn with_target(mut self, target_table: impl Into<String>, target_id: i64) -> Self {
        self.target_table = Some(target_table.into());
        self.target_id = Some(target_id);
        self
    }
}
