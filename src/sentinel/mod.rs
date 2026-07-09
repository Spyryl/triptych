pub mod build;
pub mod cache;
pub mod capsule;
pub mod fingerprint;
pub mod input;
pub mod markdown;
pub mod sha256;

pub use build::{BuildReport, CapsuleBuildResult, CapsuleBuildStatus, build_sentinel_capsules};
pub use input::{CapsuleFormat, SentinelBuildRequest};

pub type Result<T> = std::result::Result<T, SentinelError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentinelError {
    pub code: &'static str,
    pub message: String,
}

impl SentinelError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for SentinelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for SentinelError {}

impl From<std::io::Error> for SentinelError {
    fn from(error: std::io::Error) -> Self {
        Self::new("IO_ERROR", error.to_string())
    }
}
