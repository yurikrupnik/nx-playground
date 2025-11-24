use super::controller::{create_car, delete_car, get_car, get_cars, update_car};
use super::state::CarState;
use axum::{
    routing::{get, post},
    Router,
};

pub fn router<S: CarState>() -> Router<S> {
    Router::new()
        .route("/cars", get(get_cars::<S>))
        .route("/car", post(create_car::<S>))
        .route(
            "/car/{id}",
            get(get_car::<S>)
                .put(update_car::<S>)
                .delete(delete_car::<S>),
        )
}
