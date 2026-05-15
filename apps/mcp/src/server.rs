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
use axum::{body::Body, middleware, extract::Request, response::Response};
use axum::http::HeaderValue;
use tower_http::cors::CorsLayer;

use crate::{config::Config, tools::ZealotServer};

// rmcp always returns text/event-stream, but many MCP clients (LibreChat, etc.)
// only handle plain application/json responses for non-streaming operations.
// This middleware:
//   1. Injects the Accept header required by rmcp's host check.
//   2. Converts SSE responses to plain JSON by extracting the last data: event.
async fn sse_to_json_compat(mut req: Request, next: middleware::Next) -> Response {
    let has_required = req
        .headers()
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("application/json") && s.contains("text/event-stream"))
        .unwrap_or(false);
    if !has_required {
        req.headers_mut().insert(
            "accept",
            HeaderValue::from_static("application/json, text/event-stream"),
        );
    }

    let is_post = req.method() == axum::http::Method::POST;
    let is_get  = req.method() == axum::http::Method::GET;

    // LibreChat sends GET /mcp without a session ID before it has one.
    // rmcp returns 400 for this, which LibreChat treats as a fatal transport
    // error (unlike 405, which it handles gracefully). Return 405 immediately
    // so LibreChat skips the notification stream and proceeds with POST-only mode.
    if is_get {
        let has_session = req.headers().contains_key("mcp-session-id");
        if !has_session {
            return Response::builder()
                .status(405)
                .header("allow", "POST")
                .body(Body::from("Method Not Allowed"))
                .unwrap();
        }
    }

    let response = next.run(req).await;

    let is_sse = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("text/event-stream"))
        .unwrap_or(false);

    if !is_post || !is_sse {
        return response;
    }

    let (mut parts, body) = response.into_parts();
    let bytes = match axum::body::to_bytes(body, 4 * 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => return Response::from_parts(parts, Body::empty()),
    };

    let text = std::str::from_utf8(&bytes).unwrap_or("");
    let json_payload: Option<serde_json::Value> = text
        .lines()
        .filter_map(|l| l.strip_prefix("data:"))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .filter_map(|s| serde_json::from_str(s).ok())
        .last();

    if let Some(json) = json_payload {
        let json_bytes = serde_json::to_vec(&json).unwrap_or_default();
        let len = json_bytes.len();
        parts.headers.insert(
            axum::http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        parts.headers.insert(
            axum::http::header::CONTENT_LENGTH,
            HeaderValue::from_str(&len.to_string()).unwrap(),
        );
        parts.headers.remove("transfer-encoding");
        Response::from_parts(parts, Body::from(json_bytes))
    } else {
        Response::from_parts(parts, Body::from(bytes))
    }
}

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
    // Empty allowed_hosts means any Host header is permitted — required for
    // Docker deployments where clients connect via service hostname, not localhost.
    sse_config.allowed_hosts = vec![];

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
        .layer(CorsLayer::permissive())
        .layer(middleware::from_fn(sse_to_json_compat));

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!("Listening on http://0.0.0.0:{port}/mcp");
    axum::serve(listener, app).await?;
    Ok(())
}
