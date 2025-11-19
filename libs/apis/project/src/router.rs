use super::controller::{
    create_project, delete_project, delete_project_by_id, get_project, get_projects,
};
use super::state::ProjectState;
use crate::model::Model;
use axum::{routing::get, Router};
use proc_macros::ApiResource;

pub fn router<S: ProjectState>() -> Router<S> {
    Router::new()
        .route(
            Model::URL,
            get(get_projects::<S>)
                .post(create_project::<S>)
                .delete(delete_project::<S>),
        )
        .route(
            Model::URL_WITH_ID,
            get(get_project::<S>).delete(delete_project_by_id::<S>),
        )
}
