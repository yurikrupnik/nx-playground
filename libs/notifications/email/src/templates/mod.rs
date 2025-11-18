//! Email template management
//!
//! Templates can be loaded from files or stored in a database.
//! This module provides a simple in-memory template store.

use eyre::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Rendered template result
pub struct RenderedTemplate {
    pub subject: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
}

/// Template store trait
#[async_trait::async_trait]
pub trait TemplateStore: Send + Sync {
    /// Get a template by name
    async fn get(&self, name: &str) -> Result<Option<EmailTemplate>>;

    /// Store a template
    async fn set(&self, template: EmailTemplate) -> Result<()>;

    /// List all template names
    async fn list(&self) -> Result<Vec<String>>;
}

/// Email template definition
#[derive(Clone, Debug)]
pub struct EmailTemplate {
    pub name: String,
    pub subject: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
}

impl EmailTemplate {
    /// Render the template with the given data
    /// Uses simple {{variable}} replacement
    pub fn render(&self, data: &Value) -> Result<RenderedTemplate> {
        let subject = render_string(&self.subject, data)?;
        let body_text = self
            .body_text
            .as_ref()
            .map(|t| render_string(t, data))
            .transpose()?;
        let body_html = self
            .body_html
            .as_ref()
            .map(|t| render_string(t, data))
            .transpose()?;

        Ok(RenderedTemplate {
            subject,
            body_text,
            body_html,
        })
    }
}

/// Simple template variable replacement
/// Replaces {{variable}} with values from the JSON data
fn render_string(template: &str, data: &Value) -> Result<String> {
    let mut result = template.to_string();

    if let Value::Object(map) = data {
        for (key, value) in map {
            let placeholder = format!("{{{{{}}}}}", key);
            let replacement = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => String::new(),
                _ => serde_json::to_string(value).unwrap_or_default(),
            };
            result = result.replace(&placeholder, &replacement);
        }
    }

    Ok(result)
}

/// In-memory template store
pub struct InMemoryTemplateStore {
    templates: Arc<RwLock<HashMap<String, EmailTemplate>>>,
}

impl InMemoryTemplateStore {
    pub fn new() -> Self {
        Self {
            templates: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with default templates
    pub fn with_defaults() -> Self {
        let store = Self::new();
        let templates = store.templates.clone();

        // Add some default templates
        tokio::spawn(async move {
            let mut guard = templates.write().await;

            guard.insert(
                "welcome".to_string(),
                EmailTemplate {
                    name: "welcome".to_string(),
                    subject: "Welcome to {{app_name}}, {{name}}!".to_string(),
                    body_text: Some(
                        "Hello {{name}},\n\nWelcome to {{app_name}}!\n\nBest regards,\nThe Team"
                            .to_string(),
                    ),
                    body_html: Some(
                        r#"
                        <h1>Welcome, {{name}}!</h1>
                        <p>Thank you for joining {{app_name}}.</p>
                        <p>Best regards,<br>The Team</p>
                    "#
                        .to_string(),
                    ),
                },
            );

            guard.insert(
                "password_reset".to_string(),
                EmailTemplate {
                    name: "password_reset".to_string(),
                    subject: "Password Reset Request".to_string(),
                    body_text: Some("Hello {{name}},\n\nClick here to reset your password: {{reset_link}}\n\nThis link expires in {{expiry_hours}} hours.".to_string()),
                    body_html: Some(r#"
                        <h1>Password Reset</h1>
                        <p>Hello {{name}},</p>
                        <p>Click <a href="{{reset_link}}">here</a> to reset your password.</p>
                        <p>This link expires in {{expiry_hours}} hours.</p>
                    "#.to_string()),
                },
            );
        });

        store
    }
}

impl Default for InMemoryTemplateStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl TemplateStore for InMemoryTemplateStore {
    async fn get(&self, name: &str) -> Result<Option<EmailTemplate>> {
        let guard = self.templates.read().await;
        Ok(guard.get(name).cloned())
    }

    async fn set(&self, template: EmailTemplate) -> Result<()> {
        let mut guard = self.templates.write().await;
        guard.insert(template.name.clone(), template);
        Ok(())
    }

    async fn list(&self) -> Result<Vec<String>> {
        let guard = self.templates.read().await;
        Ok(guard.keys().cloned().collect())
    }
}
