mod account;
mod health;

use axum::Router;
use zealot_app::app::AppState;

pub fn build_router(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/health", health::routes())
        .with_state(state)
}