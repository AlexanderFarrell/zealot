use axum::{
    extract::{Query, State},
    middleware,
    routing::get,
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use zealot_app::{app::AppState, services::analysis::AnalysisServiceError};
use zealot_domain::auth::Actor;

use crate::http::{
    common::HttpError,
    middleware::{auth_middleware, csrf_middleware},
    scope::{read_scope_access, ScopeQuery},
};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/most-viewed", get(get_most_viewed))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            csrf_middleware,
        ))
        .route_layer(middleware::map_request_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state)
}

fn analysis_service_err(err: AnalysisServiceError) -> HttpError {
    match err {
        AnalysisServiceError::Repo(e) => {
            tracing::error!("Analysis repo error: {e}");
            HttpError::Internal
        }
    }
}

#[derive(Deserialize)]
struct MostViewedParams {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(flatten)]
    scope: ScopeQuery,
}

fn default_limit() -> i64 {
    30
}

#[derive(Serialize)]
pub struct MostViewedItemDto {
    pub item_id: i64,
    pub title: String,
    pub view_count: i64,
}

async fn get_most_viewed(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Query(params): Query<MostViewedParams>,
) -> Result<Json<Vec<MostViewedItemDto>>, HttpError> {
    let access = read_scope_access(&state, &actor, &params.scope)?;
    let results = state
        .services
        .analysis
        .get_most_viewed_items_in_scopes(params.limit, &access)
        .map_err(analysis_service_err)?;

    let dtos = results
        .into_iter()
        .map(|r| MostViewedItemDto {
            item_id: r.item.item_id.into(),
            title: r.item.title,
            view_count: r.view_count,
        })
        .collect();

    Ok(Json(dtos))
}
