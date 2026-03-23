use axum::{Router, routing::get};
use zealot_app::app::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(health))
        .route("/ready", get(readiness))
}

async fn health() -> &'static str {
    "ok"
}

async fn readiness() -> &'static str {
    "ready"
}
