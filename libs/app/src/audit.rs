use axum::http::HeaderMap;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::net::SocketAddr;

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditOutcome {
    Success,
    Failure,
    Denied,
}

#[derive(Debug, Serialize)]
pub struct AuditEvent {
    pub user_id: Option<String>,
    pub action: String,
    pub resource: Option<String>,
    pub outcome: AuditOutcome,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
    pub details: Option<serde_json::Value>,
}

impl AuditEvent {
    pub fn new(
        user_id: Option<String>,
        action: impl Into<String>,
        resource: Option<String>,
        outcome: AuditOutcome,
    ) -> Self {
        Self {
            user_id,
            action: action.into(),
            resource,
            outcome,
            ip_address: None,
            user_agent: None,
            timestamp: Utc::now(),
            details: None,
        }
    }

    pub fn with_ip(mut self, ip: Option<String>) -> Self {
        self.ip_address = ip;
        self
    }

    pub fn with_user_agent(mut self, user_agent: Option<String>) -> Self {
        self.user_agent = user_agent;
        self
    }

    pub fn with_details(mut self, details: impl Serialize) -> Self {
        self.details = serde_json::to_value(details).ok();
        self
    }

    pub fn log(self) {
        tracing::info!(
            target: "audit",
            user_id = self.user_id,
            action = %self.action,
            resource = self.resource,
            outcome = ?self.outcome,
            ip = self.ip_address,
            user_agent = self.user_agent,
            timestamp = %self.timestamp,
            details = ?self.details,
            "{}",
            serde_json::to_string(&self).unwrap_or_else(|_| "Failed to serialize audit event".to_string())
        );
    }
}

pub fn extract_ip_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
}

pub fn extract_ip_from_socket(socket: Option<SocketAddr>) -> Option<String> {
    socket.map(|addr| addr.ip().to_string())
}

pub fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}
