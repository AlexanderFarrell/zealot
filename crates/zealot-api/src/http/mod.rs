mod account;
mod attribute;
mod auth;
mod comment;
pub mod common;
mod health;
mod item;
mod item_type;
mod media;
mod middleware;
mod planner;
mod repeat;
mod rule;

use axum::Router;
use axum::http::HeaderValue;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use zealot_app::{app::AppState, config::ZealotConfig};

pub async fn run_http(state: AppState, config: ZealotConfig) -> Result<(), String> {
    let router = build_router(state);
    let address = config.get_host_port();
    let listener = tokio::net::TcpListener::bind(address.clone())
        .await
        .map_err(|e| format!("Failed to listen at {}: {}", &address, e))?;

    axum::serve(listener, router)
        .await
        .map_err(|_e| format!("Http server failed to listen"))
}

fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin("tauri://localhost".parse::<HeaderValue>().unwrap())
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(true);

    Router::new()
        .nest("/health", health::routes())
        .nest("/account", account::routes(state.clone()))
        .nest("/auth", auth::routes(state.clone()))
        .nest("/comment", comment::routes(state.clone()))
        .nest("/item", item::routes(state.clone()))
        .nest("/item_type", item_type::routes(state.clone()))
        .nest("/attribute", attribute::routes(state.clone()))
        .nest("/media", media::routes(state.clone()))
        .nest("/planner", planner::routes(state.clone()))
        .nest("/repeat", repeat::routes(state.clone()))
        .nest("/rule", rule::routes(state.clone()))
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(state)
}
