use crate::app_state::AppState;
use auth::router::router as auth_router;
use axum::{routing::{get, post}, Router};
use project::router::router as project_router;
use std::time::Duration;
use streaming::router::router as streaming_router;
use task::router::router as task_router;
use tokio::time::sleep;

pub mod auth;
pub mod feature_flags_api;
pub mod oauth;
pub mod streaming;
pub mod task;

pub fn routes(state: &AppState) -> Router<AppState> {
  let mut router = Router::new()
    .nest("/streaming", streaming_router())
    .merge(task_router())
    .merge(project_router::<AppState>());

  let mut auth_routes = auth_router();

  if state.oauth_config.is_some() {
    auth_routes = auth_routes.nest("/oauth", crate::oauth_router::oauth_router());
  }

  router = router.nest("/auth", auth_routes);

  if state.feature_flags.is_some() {
    router = router.route("/features/check", post(feature_flags_api::check_feature));
  }

  router
}
