//! SMTP email provider using lettre

use super::{EmailProvider, SendResult};
use crate::models::Email;
use async_trait::async_trait;
use eyre::{Result, WrapErr};
use lettre::{
    message::{header::ContentType, Mailbox, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use std::sync::Arc;

/// SMTP provider configuration
#[derive(Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub from_name: String,
    pub use_tls: bool,
}

/// SMTP email provider
pub struct SmtpProvider {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    config: Arc<SmtpConfig>,
}

impl SmtpProvider {
    /// Create a new SMTP provider
    pub fn new(config: SmtpConfig) -> Result<Self> {
        let creds = Credentials::new(config.username.clone(), config.password.clone());

        let transport = if config.use_tls {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
                .wrap_err("Failed to create SMTP relay")?
                .credentials(creds)
                .port(config.port)
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)
                .credentials(creds)
                .port(config.port)
                .build()
        };

        Ok(Self {
            transport,
            config: Arc::new(config),
        })
    }

    fn build_message(&self, email: &Email) -> Result<Message> {
        let from: Mailbox = format!("{} <{}>", self.config.from_name, self.config.from_email)
            .parse()
            .wrap_err("Invalid from address")?;

        let to: Mailbox = email.to.parse().wrap_err("Invalid to address")?;

        let mut builder = Message::builder().from(from).to(to).subject(&email.subject);

        // Add reply-to if specified
        if let Some(reply_to) = &email.reply_to {
            let reply_to_mailbox: Mailbox =
                reply_to.parse().wrap_err("Invalid reply-to address")?;
            builder = builder.reply_to(reply_to_mailbox);
        }

        // Add CC recipients
        for cc in &email.cc {
            let cc_mailbox: Mailbox = cc.parse().wrap_err("Invalid CC address")?;
            builder = builder.cc(cc_mailbox);
        }

        // Add BCC recipients
        for bcc in &email.bcc {
            let bcc_mailbox: Mailbox = bcc.parse().wrap_err("Invalid BCC address")?;
            builder = builder.bcc(bcc_mailbox);
        }

        // Build body
        let message = match (&email.body_text, &email.body_html) {
            (Some(text), Some(html)) => {
                // Multipart message with both text and HTML
                builder
                    .multipart(
                        MultiPart::alternative()
                            .singlepart(
                                SinglePart::builder()
                                    .header(ContentType::TEXT_PLAIN)
                                    .body(text.clone()),
                            )
                            .singlepart(
                                SinglePart::builder()
                                    .header(ContentType::TEXT_HTML)
                                    .body(html.clone()),
                            ),
                    )
                    .wrap_err("Failed to build multipart message")?
            }
            (Some(text), None) => builder
                .header(ContentType::TEXT_PLAIN)
                .body(text.clone())
                .wrap_err("Failed to build text message")?,
            (None, Some(html)) => builder
                .header(ContentType::TEXT_HTML)
                .body(html.clone())
                .wrap_err("Failed to build HTML message")?,
            (None, None) => {
                return Err(eyre::eyre!("Email must have either text or HTML body"));
            }
        };

        Ok(message)
    }
}

#[async_trait]
impl EmailProvider for SmtpProvider {
    async fn send(&self, email: &Email) -> Result<SendResult> {
        let message = self.build_message(email)?;

        let response = self
            .transport
            .send(message)
            .await
            .wrap_err("Failed to send email via SMTP")?;

        // Extract message ID from response
        let message_id = response
            .message()
            .next()
            .map(|s| s.to_string())
            .unwrap_or_else(|| email.id.clone());

        tracing::info!(
            email_id = %email.id,
            to = %email.to,
            subject = %email.subject,
            "Email sent successfully"
        );

        Ok(SendResult { message_id })
    }

    async fn health_check(&self) -> Result<()> {
        self.transport
            .test_connection()
            .await
            .wrap_err("SMTP health check failed")?;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "smtp"
    }
}
