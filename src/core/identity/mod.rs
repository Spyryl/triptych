use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdentityContext {
    pub user_id: i64,
    pub user_code: String,
    pub user_type: String,
    pub token_id: Option<i64>,
    pub authority_user_id: Option<i64>,
    pub project_id: Option<i64>,
    pub request_id: Uuid,
    pub source_tool: String,
}

impl IdentityContext {
    pub fn bootstrap_cli() -> Self {
        Self {
            user_id: 0,
            user_code: "bootstrap-cli".to_string(),
            user_type: "service".to_string(),
            token_id: None,
            authority_user_id: None,
            project_id: None,
            request_id: Uuid::new_v4(),
            source_tool: "mandate-cli".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScopeRequirement {
    pub scope_code: String,
    pub project_id: Option<i64>,
}

impl ScopeRequirement {
    pub fn new(scope_code: impl Into<String>, project_id: Option<i64>) -> Self {
        Self {
            scope_code: scope_code.into(),
            project_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorisationDecision {
    pub allowed: bool,
    pub reason: String,
}

impl AuthorisationDecision {
    pub fn allow(reason: impl Into<String>) -> Self {
        Self {
            allowed: true,
            reason: reason.into(),
        }
    }

    pub fn deny(reason: impl Into<String>) -> Self {
        Self {
            allowed: false,
            reason: reason.into(),
        }
    }
}
