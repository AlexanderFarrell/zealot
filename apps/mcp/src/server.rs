use std::sync::Arc;
use std::time::Duration;

use rmcp::ServiceExt;
use rmcp::transport::stdio;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig,
    session::local::LocalSessionManager,
    tower::StreamableHttpService,
};
use axum::routing::get;
use tower_http::cors::CorsLayer;

use crate::{config::Config, tools::ZealotServer};

pub async fn run_stdio(config: Config) -> anyhow::Result<()> {
    tracing::info!("Starting Zealot MCP server (stdio mode)");
    let service = ZealotServer::new(&config);
    let server = service
        .serve(stdio())
        .await
        .map_err(|e| anyhow::anyhow!("MCP serve error: {e}"))?;
    server
        .waiting()
        .await
        .map_err(|e| anyhow::anyhow!("MCP server error: {e}"))?;
    Ok(())
}

pub async fn run_http(config: Config) -> anyhow::Result<()> {
    let port = config.port;
    tracing::info!("Starting Zealot MCP server (HTTP mode) on port {port}");

    let cfg = Arc::new(config);
    let session_manager = Arc::new(LocalSessionManager::default());

    let mut sse_config = StreamableHttpServerConfig::default();
    sse_config.sse_keep_alive = Some(Duration::from_secs(15));
    // allow connections from any host when running as a public HTTP server
    sse_config.allowed_hosts = vec![
        "localhost".into(),
        "127.0.0.1".into(),
        "0.0.0.0".into(),
        "::1".into(),
    ];

    let svc = {
        let cfg = cfg.clone();
        StreamableHttpService::new(
            move || Ok(ZealotServer::new(&cfg)),
            session_manager,
            sse_config,
        )
    };

    let app = axum::Router::new()
        .route_service("/mcp", svc)
        .route("/health", get(|| async { "ok" }))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!("Listening on http://0.0.0.0:{port}/mcp");
    axum::serve(listener, app).await?;
    Ok(())
}
