use serde::{Deserialize, Serialize};

/// Email priority levels for queue processing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EmailPriority {
    /// Urgent emails (password reset, OTP)
    High,
    /// Normal transactional emails
    Normal,
    /// Bulk/marketing emails
    Low,
}

impl Default for EmailPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Email processing status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EmailStatus {
    Pending,
    Processing,
    Sent,
    Failed,
    Retrying,
}

/// Email message to be sent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    /// Unique identifier for the email
    pub id: String,
    /// Recipient email address
    pub to: String,
    /// Optional CC recipients
    #[serde(default)]
    pub cc: Vec<String>,
    /// Optional BCC recipients
    #[serde(default)]
    pub bcc: Vec<String>,
    /// Email subject
    pub subject: String,
    /// Plain text body
    pub body_text: Option<String>,
    /// HTML body
    pub body_html: Option<String>,
    /// Sender email (defaults to configured from address)
    pub from: Option<String>,
    /// Reply-to address
    pub reply_to: Option<String>,
    /// Email priority
    #[serde(default)]
    pub priority: EmailPriority,
    /// Template name (if using templates)
    pub template: Option<String>,
    /// Template variables
    #[serde(default)]
    pub template_data: serde_json::Value,
    /// Retry count
    #[serde(default)]
    pub retry_count: u32,
    /// Maximum retries allowed
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

fn default_max_retries() -> u32 {
    3
}

impl Email {
    /// Create a new email with required fields
    pub fn new(to: impl Into<String>, subject: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            to: to.into(),
            cc: Vec::new(),
            bcc: Vec::new(),
            subject: subject.into(),
            body_text: None,
            body_html: None,
            from: None,
            reply_to: None,
            priority: EmailPriority::Normal,
            template: None,
            template_data: serde_json::Value::Null,
            retry_count: 0,
            max_retries: 3,
        }
    }

    /// Set plain text body
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.body_text = Some(text.into());
        self
    }

    /// Set HTML body
    pub fn with_html(mut self, html: impl Into<String>) -> Self {
        self.body_html = Some(html.into());
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: EmailPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set template
    pub fn with_template(mut self, name: impl Into<String>, data: serde_json::Value) -> Self {
        self.template = Some(name.into());
        self.template_data = data;
        self
    }

    /// Check if email can be retried
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    /// Increment retry count
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}

/// Event types for the email stream
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "payload")]
pub enum EmailEvent {
    /// Request to send an email
    SendEmail(Email),
    /// Email was sent successfully
    EmailSent { id: String, message_id: String },
    /// Email sending failed
    EmailFailed {
        id: String,
        error: String,
        retryable: bool,
    },
}
