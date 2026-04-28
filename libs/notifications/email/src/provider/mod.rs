//! Email provider implementations

pub mod smtp;

pub use smtp::{SmtpConfig, SmtpProvider};

use crate::models::Email;
use async_trait::async_trait;
use eyre::Result;

/// Result of sending an email
#[derive(Debug)]
pub struct SendResult {
    /// Provider-specific message ID
    pub message_id: String,
}

/// Trait for email providers
#[async_trait]
pub trait EmailProvider: Send + Sync {
    /// Send an email
    async fn send(&self, email: &Email) -> Result<SendResult>;

    /// Check if the provider is healthy
    async fn health_check(&self) -> Result<()>;

    /// Get provider name
    fn name(&self) -> &'static str;
}
