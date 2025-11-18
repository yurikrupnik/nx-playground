use email::provider::SmtpProvider;
use email::templates::{InMemoryTemplateStore, TemplateStore};
use redis::aio::ConnectionManager;
use std::sync::Arc;

/// Application state for the email worker
#[derive(Clone)]
pub struct AppState {
    pub redis: ConnectionManager,
    pub provider: Arc<SmtpProvider>,
    pub templates: Arc<dyn TemplateStore>,
}

impl AppState {
    pub fn new(
        redis: ConnectionManager,
        provider: SmtpProvider,
        templates: impl TemplateStore + 'static,
    ) -> Self {
        Self {
            redis,
            provider: Arc::new(provider),
            templates: Arc::new(templates),
        }
    }

    /// Create with default template store
    pub fn with_defaults(redis: ConnectionManager, provider: SmtpProvider) -> Self {
        Self::new(redis, provider, InMemoryTemplateStore::with_defaults())
    }
}

// Implement FromRef for extracting parts of the state
impl axum::extract::FromRef<AppState> for ConnectionManager {
    fn from_ref(state: &AppState) -> Self {
        state.redis.clone()
    }
}
