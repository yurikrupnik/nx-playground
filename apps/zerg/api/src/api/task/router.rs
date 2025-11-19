use super::controller::{create_task, delete_task, drop_tasks, get_task, get_tasks, update_task};
use super::model::Task;
use crate::app_state::AppState;
use axum::{routing, Router};
use proc_macros::DbResource;

/// Task router - includes full CRUD
pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            Task::URL,
            routing::get(get_tasks).delete(drop_tasks).post(create_task),
        )
        .route(
            Task::URL_WITH_ID,
            routing::get(get_task).put(update_task).delete(delete_task),
        )
}
