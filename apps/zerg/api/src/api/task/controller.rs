use super::model::{CreateTask, Task, UpdateTask};
use crate::app_state::AppState;
use app::errors::{ApiErrorMessage, AppError as ServerError};
use app::extractors::{UuidPath, ValidatedJson};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use events::{publish_event, TaskEvent};
use services::postgres::results::{handle_delete_result, handle_drop_result, handle_result};
use services::postgres::service::SqlMethods;
use tracing::info;

/// Get list of tasks.
///
/// List `Task`.
///
/// One could call the api endpoint with following curl.
/// ```text
/// curl localhost:8080/api/task
/// ```
#[utoipa::path(
  get,
  path = "/api/task",
  tag = "Tasks",
  responses(
(status = 200, description = "Collection found successfully", body = [Task]),
    // (status = 400, description = "Api error", body = ErrorResponse),
    // (status = 500, description = "Internal error", body = ErrorResponse),
  ),
)]
pub async fn get_tasks(State(app_state): State<AppState>) -> impl IntoResponse {
    let result = Task::get_list(&app_state.pool, &None).await;
    // let result = get_list::<Task>(&app_state.pool, Some(query)).await;
    handle_result(result, StatusCode::OK)
}

/// Get Task by id.
///
/// Return found `Task` with status 200 or 404 not found if `Task` is not found from Postgres DB.
#[utoipa::path(
  get,
  path = "/api/task/{id}",
  tag = "Tasks",
  responses(
(status = 200, description = "Task found from db", body = Task),
    // (status = 404, description = "Task not found by id", body = ErrorResponse, example = json!(ErrorResponse::NotFound(String::from("id = 1"))))
  ),
  params(
("id", description = "Unique storage id of Task")
  )
)]
pub async fn get_task(
    UuidPath(id): UuidPath,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    info!("Get task by id: {}", &id);
    let result = Task::get_by_id(&app_state.pool, &id).await;
    // let result = get_by_id::<Task>(&app_state.pool, &id).await;
    handle_result(result, StatusCode::OK)
}

/// Delete Task by given path variable id.
///
/// This ednpoint needs `api_key` authentication in order to call. Api key can be found from README.md.
///
/// Api will delete `Task` by the provided id and return success 200.
/// If storage does not contain `Task` with given id 404 not found will be returned.
#[utoipa::path(
  delete,
  path = "/api/task/{id}",
  tag = "Tasks",
  responses(
(status = 200, description = "Task deleted successfully"),
(status = 403, description = "Missing or invalid CSRF token"),
  ),
  params(
("id", description = "Unique id of Task")
  ),
  security(
("csrf_token" = [])
  )
)]
pub async fn delete_task(UuidPath(id): UuidPath, app_state: State<AppState>) -> impl IntoResponse {
    let result = Task::delete_by_id(&app_state.pool, &id).await;

    if result.is_ok() {
        let event = TaskEvent::Deleted {
            id,
            deleted_at: Utc::now(),
        };
        publish_event(&app_state.redis, &event).await;
    }

    handle_delete_result(result, &id.to_string())
}

/// Create new Task.
///
/// Post a new `Task` in request body as json to store it. Api will return
/// created `Task` on success.
///
/// One could call the api with following curl.
/// ```text
/// curl -X POST -H "Content-Type: application/json" -d '{"firstName": "Test name", "lastName": "Test last", "email": "a@a.com", "username": "test"}' localhost:8080/api/task
/// ```
#[utoipa::path(
post,
path = "/api/task",
tag = "Tasks",
request_body = CreateTask,
responses(
(status = 201, description = "Task created successfully", body = Task),
(status = 403, description = "Missing or invalid CSRF token"),
),
security(
("csrf_token" = [])
)
)]
pub async fn create_task(
    app_state: State<AppState>,
    ValidatedJson(body): ValidatedJson<CreateTask>,
) -> impl IntoResponse {
    let result = Task::create_item(&app_state.pool, &body).await;

    match result {
        Ok(task) => {
            let event = TaskEvent::Created(task.clone());
            publish_event(&app_state.redis, &event).await;
            (StatusCode::CREATED, Json(task)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create task: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response()
        }
    }
}

/// Drop Task collection.
///
/// Api will delete all `Task` and return success 200.
/// If storage does not contain `Task` with given id 404 not found will be returned.
#[utoipa::path(
  delete,
  path = "/api/task",
  tag = "Tasks",
  responses(
(status = 200, description = "Task deleted successfully"),
(status = 403, description = "Missing or invalid CSRF token"),
  ),
  security(
("csrf_token" = [])
  )
)]
pub async fn drop_tasks(app_state: State<AppState>) -> impl IntoResponse {
    let result = Task::drop_collection(&app_state.pool).await;
    handle_drop_result(result)
}

/// Update Task with given id.
///
/// This endpoint supports optional authentication.
///
/// Tries to update `Task` by given id as path variable. If todo is found by id values are
/// updated according `UpdateTask` and updated `Task` is returned with status 200.
/// If todo is not found then 404 not found is returned.
#[utoipa::path(
put,
path = "/api/task/{id}",
tag = "Tasks",
request_body = UpdateTask,
responses(
(status = 200, description = "Success", body = Task),
(
    status = 400,
    description = "Bad Request",
    body = ApiErrorMessage,
    example = json!({
        "status": 400,
        "error": "Validation Error",
        "message": "Validation failed for the provided input.",
        "details": {
            "title": [
                {
                    "code": "length",
                    "message": "Title must be at least 2 characters long",
                    "params": {
                        "min": 2,
                        "value": "title"
                    }
                }
            ]
        }
    })
),
(
    status = 404,
    description = "Task not found",
    body = ApiErrorMessage,
    example = json!({
        "status": 404,
        "error": "Not Found",
        "message": "Task with ID 00000000-0000-0000-0000-000000000000 not found",
        "details": null
    })
),
(
    status = 500,
    description = "Internal Server Error",
    body = ApiErrorMessage,
    example = json!({
        "status": 500,
        "error": "Internal Server Error",
        "message": "An unexpected error occurred.",
        "details": null
    })
),
(status = 403, description = "Missing or invalid CSRF token"),
),
params(
("id", description = "Unique storage id of Task")
),
security(
("csrf_token" = [])
)
)]
pub async fn update_task(
    app_state: State<AppState>,
    UuidPath(id): UuidPath,
    ValidatedJson(body): ValidatedJson<UpdateTask>,
) -> Result<Json<Task>, ServerError> {
    let result = Task::update_by_id(&app_state.pool, &id, &body).await?;

    let event = if result.completed {
        TaskEvent::Completed {
            id: result.id,
            completed_at: Utc::now(),
        }
    } else {
        TaskEvent::Updated(result.clone())
    };

    publish_event(&app_state.redis, &event).await;

    Ok(Json(result))
}
