//! Email notification library with Redis Streams support
//!
//! This library provides:
//! - Email models and DTOs
//! - Redis Streams integration for event-driven processing
//! - Email provider abstractions (SMTP, etc.)
//! - Template management

pub mod models;
pub mod provider;
pub mod stream;
pub mod templates;

pub use models::{Email, EmailPriority, EmailStatus};
pub use provider::EmailProvider;
pub use stream::{EmailConsumer, EmailProducer};
