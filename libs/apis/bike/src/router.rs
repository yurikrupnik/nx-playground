use super::controller::{create_bike, delete_bike, get_bike, get_bikes, update_bike};
use super::state::BikeState;
use axum::{
    routing::{get, post},
    Router,
};

pub fn router<S: BikeState>() -> Router<S> {
    Router::new()
        .route("/bikes", get(get_bikes::<S>))
        .route("/bike", post(create_bike::<S>))
        .route(
            "/bike/{id}",
            get(get_bike::<S>)
                .put(update_bike::<S>)
                .delete(delete_bike::<S>),
        )
}
