use super::model::{
    ActiveModel as ProjectActiveModel, CreateProjectDto, Entity as Project, ProjectResponseDto,
};
use super::state::ProjectState;
use app::{
    errors::AppError,
    extractors::{UuidPath, ValidatedJson},
    responses::{
        BadRequestUuidResponse, BadRequestValidationResponse, InternalServerErrorResponse,
        NotFoundResponse,
    },
};
use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
};
use chrono::Utc;
use field_selector::{AuthContext, FieldSelector};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, Set};
use serde_json::Value;
use uuid::Uuid;
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
        (status = 200, description = "List of all projects retrieved successfully", body = [ProjectResponseDto]),
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

    let projects = query.all(state.pool()).await?;
    let response: Vec<ProjectResponseDto> = projects.into_iter().map(|p| p.into()).collect();

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
    request_body = CreateProjectDto,
    responses(
        (status = 201, description = "Project created successfully", body = ProjectResponseDto),
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
    ValidatedJson(body): ValidatedJson<CreateProjectDto>,
) -> Result<(StatusCode, Json<ProjectResponseDto>), AppError> {
    let project = ProjectActiveModel {
        id: Set(Uuid::now_v7()),
        title: Set(body.title),
        description: Set(body.description),
        completed: Set(body.completed),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    };

    let project = project.insert(state.pool()).await?;
    // let project = Project::create_item(state.pool(), &body).await?;
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
    Project::delete_many().exec(state.pool()).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/project/{id}",
    tag = "Projects",
    params(
        ("id", description = "Unique UUID of the project")
    ),
    responses(
        (status = 200, description = "Project found successfully", body = ProjectResponseDto),
        (status = 400, response = BadRequestUuidResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse),
    ),
)]
pub async fn get_project<S: ProjectState>(
    State(state): State<S>,
    UuidPath(id): UuidPath,
    Query(field_selector): Query<FieldSelector>,
) -> Result<(StatusCode, Json<ProjectResponseDto>), AppError> {
    // let result = Project::find_by_id(id).one(state.pool()).await?;
    let project = Project::find_by_id(id)
        .one(state.pool())
        .await?
        .ok_or(AppError::NotFound(format!(
            "Project with id {} not found",
            id
        )))?;

    let response: ProjectResponseDto = project.into();

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
        ("id", description = "Unique UUID of the project")
    ),
    responses(
        (status = 200, description = "Project deleted successfully"),
        (status = 400, response = BadRequestUuidResponse),
        (status = 404, response = NotFoundResponse),
        (status = 500, response = InternalServerErrorResponse),
    ),
)]
pub async fn delete_project_by_id<S: ProjectState>(
    State(state): State<S>,
    UuidPath(id): UuidPath,
) -> Result<StatusCode, AppError> {
    let result = Project::delete_by_id(id).exec(state.pool()).await?;

    if result.rows_affected == 0 {
        return Err(AppError::NotFound(format!(
            "Project with id {} not found",
            id
        )));
    }

    Ok(StatusCode::OK)
}
