mod auth;
mod health;
pub mod common;
mod middleware;

use axum::Router;
use zealot_app::{
    app::{self, AppState},
    config::ZealotConfig,
};

pub async fn run_http(state: AppState, config: ZealotConfig) -> Result<(), String> {
    let router = build_router(state);
    let address = config.get_host_port();
    let listener = tokio::net::TcpListener::bind(address.clone())
        .await
        .map_err(|e| format!("Failed to listen at {}: {}", &address, e))?;

    axum::serve(listener, router)
        .await
        .map_err(|e| format!("Http server failed to listen"))
}

fn build_router(state: AppState) -> Router {
    Router::new()
        .nest("/health", health::routes())
        .with_state(state)
}
