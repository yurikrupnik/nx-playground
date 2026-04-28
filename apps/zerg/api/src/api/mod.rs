use crate::state::AppState;
use apis_bike::router::router as bike_router;
use apis_car::router::router as car_router;
use apis_project::router::router as project_router;
use axum::Router;

// pub mod auth;
// pub mod feature_flags_api;
// pub mod oauth;
// pub mod streaming;
// pub mod task; // TODO: Fix broken imports

/// Creates the API router with all API routes
/// Used with the generic create_router function from libs/app
pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(project_router::<AppState>())
        .merge(car_router::<AppState>())
        .merge(bike_router::<AppState>())
    // Add more routers here as they're implemented:
    // .merge(task_router::<AppState>())
    // .nest("/streaming", streaming_router())
}
