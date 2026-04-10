use axum::{Extension, Json, Router, extract::State, http::StatusCode, middleware, routing::patch};
use zealot_app::app::AppState;
use zealot_domain::auth::Actor;

use crate::http::{common::HttpError, middleware::auth_middleware};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/settings", patch(update_settings))
        .route_layer(middleware::map_request_with_state(state.clone(), auth_middleware))
        .with_state(state)
}

async fn update_settings(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(settings): Json<serde_json::Value>,
) -> Result<StatusCode, HttpError> {
    if !actor.is_authenticated() {
        return Err(HttpError::Unauthorized);
    }
    let account = actor.account.ok_or(HttpError::Unauthorized)?;
    state.services.account
        .update_settings(&account.account_id, settings)
        .map_err(|_| HttpError::Internal)?;
    Ok(StatusCode::OK)
}
