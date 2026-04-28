use super::model::{
    ActiveModel as ProjectActiveModel, CreateProject, Entity as Project, ProjectResponse,
};
use super::state::ProjectState;
use app::{
    errors::AppError,
    extractors::ValidatedJson,
    responses::{BadRequestValidationResponse, InternalServerErrorResponse, NotFoundResponse},
};
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
};
use chrono::Utc;
use field_selector::{AuthContext, FieldSelector};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, NotSet, Set};
use serde_json::Value;
// use events::{publish_event, ProjectEvent};
// use services::postgres::service::SqlMethods;

#[derive(serde::Deserialize)]
pub struct ListProjectsParams {
    #[serde(flatten)]
    pub field_selector: FieldSelector,
    pub completed: Option<bool>,
}

#[utoipa::path(
    get,
    path = "/api/projects",
    tag = "Projects",
    params(
        ("fields" = Option<String>, Query, description = "Comma-separated list of fields to include"),
        ("completed" = Option<bool>, Query, description = "Filter by completed status")
    ),
    responses(
        (status = 200, description = "List of all projects retrieved successfully", body = [ProjectResponse]),
        (status = 500, response = InternalServerErrorResponse),
    ),
)]
pub async fn get_projects<S: ProjectState>(
    State(state): State<S>,
    auth: AuthContext,
    Query(params): Query<ListProjectsParams>,
) -> Result<Json<Value>, AppError> {
    use super::model::Column;

    let mut query = Project::find();

    // Filter by completed if specified
    if let Some(completed) = params.completed {
        query = query.filter(Column::Completed.eq(completed));
    }

    let projects = query.all(state.db()).await?;
    let response: Vec<ProjectResponse> = projects.into_iter().map(|p| p.into()).collect();

    let filtered = params
        .field_selector
        .filter_list_secure(&response, &auth)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(filtered))
}

#[utoipa::path(
    post,
    path = "/api/project",
    tag = "Projects",
    request_body = CreateProject,
    responses(
        (status = 201, description = "Project created successfully", body = ProjectResponse),
        (status = 400, response = BadRequestValidationResponse),
        (status = 403, description = "Missing or invalid CSRF token"),
        (status = 500, response = InternalServerErrorResponse),
    ),
    security(
        ("csrf_token" = [])
    )
)]
pub async fn create_project<S: ProjectState>(
    State(state): State<S>,
    ValidatedJson(body): ValidatedJson<CreateProject>,
) -> Result<(StatusCode, Json<ProjectResponse>), AppError> {
    let project = ProjectActiveModel {
        id: NotSet,
        title: Set(body.title),
        description: Set(body.description),
        completed: Set(body.completed),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    };

    let project = project.insert(state.db()).await?;
    // let project = Project::create_item(state.db(), &body).await?;
    //
    // let event = ProjectEvent::Created(project.clone());
    // publish_event(state.redis(), &event).await;
    //
    let response = project.into();
    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    delete,
    path = "/api/project",
    tag = "Projects",
    responses(
        (status = 200, description = "All projects deleted successfully"),
        (status = 403, description = "Missing or invalid CSRF token"),
        (status = 500, response = InternalServerErrorResponse),
    ),
    security(
        ("csrf_token" = [])
    )
)]
pub async fn delete_project<S: ProjectState>(
    State(state): State<S>,
) -> Result<StatusCode, AppError> {
    Project::delete_many().exec(state.db()).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/project/{id}",
    tag = "Projects",
    params(
        ("id", description = "Unique ID of the project")
    ),
    responses(
        (status = 200, description = "Project found successfully", body = ProjectResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse),
    ),
)]
pub async fn get_project<S: ProjectState>(
    State(state): State<S>,
    Path(id): Path<i64>,
    Query(field_selector): Query<FieldSelector>,
) -> Result<(StatusCode, Json<ProjectResponse>), AppError> {
    // let result = Project::find_by_id(id).one(state.db()).await?;
    let project = Project::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or(AppError::NotFound(format!(
            "Project with id {} not found",
            id
        )))?;

    let response: ProjectResponse = project.into();

    // let filtered = field_selector
    //   .filter_secure(&response, &auth)
    //   .map_err(|e| AppError::Validation(e.to_string()))?;

    // Ok(Json(response))
    Ok((StatusCode::OK, Json(response)))
}

#[utoipa::path(
    delete,
    path = "/api/project/{id}",
    tag = "Projects",
    params(
        ("id", description = "Unique ID of the project")
    ),
    responses(
        (status = 200, description = "Project deleted successfully"),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse),
    ),
)]
pub async fn delete_project_by_id<S: ProjectState>(
    State(state): State<S>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let result = Project::delete_by_id(id).exec(state.db()).await?;

    if result.rows_affected == 0 {
        return Err(AppError::NotFound(format!(
            "Project with id {} not found",
            id
        )));
    }

    Ok(StatusCode::OK)
}
