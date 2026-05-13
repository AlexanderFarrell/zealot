# MCP-007: HTTP + SSE Transport

## Goal

Complete the `run_http` function in `apps/mcp/src/server.rs` to serve the MCP protocol over HTTP with Server-Sent Events. This enables web-based MCP clients, containerized deployments, and the MCP Inspector tool for interactive debugging.

## Depends On

MCP-001 through MCP-006 (the service must have all tools and prompts registered)

## Background

The MCP HTTP transport uses two HTTP endpoints:
- `POST /mcp` — client sends JSON-RPC requests
- `GET /mcp` — server sends JSON-RPC responses as Server-Sent Events

`rmcp`'s `StreamableHttpService` handles both. It wraps the `ZealotServer` and manages session state via a `LocalSessionManager`. Each connected client gets an isolated session.

## File: `apps/mcp/src/server.rs`

```rust
use std::sync::Arc;
use std::time::Duration;

use axum::routing::{any, get};
use tower_http::cors::CorsLayer;

use rmcp::ServiceExt;
use rmcp::transport::stdio;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig,
    tower::{StreamableHttpService, LocalSessionManager},
};

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

    let sse_config = StreamableHttpServerConfig {
        sse_keep_alive: Some(Duration::from_secs(15)),
        ..Default::default()
    };

    let svc = {
        let cfg = cfg.clone();
        Arc::new(StreamableHttpService::new(
            move || Ok(ZealotServer::new(&cfg)),
            session_manager,
            sse_config,
        ))
    };

    let app = axum::Router::new()
        .route(
            "/mcp",
            any({
                let svc = svc.clone();
                move |req: axum::extract::Request| {
                    let svc = svc.clone();
                    async move { svc.call(req).await }
                }
            }),
        )
        .route("/health", get(|| async { "ok" }))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!("Listening on http://0.0.0.0:{port}/mcp");
    axum::serve(listener, app).await?;
    Ok(())
}
```

## Claude Desktop Configuration

For **stdio mode** (recommended for Claude Desktop):

```json
{
  "mcpServers": {
    "zealot": {
      "command": "/path/to/zealot-mcp",
      "args": ["--mode", "stdio"],
      "env": {
        "ZEALOT_URL": "http://localhost:7377",
        "ZEALOT_API_KEY": "your-api-key-here"
      }
    }
  }
}
```

For **HTTP mode** (Claude.ai remote MCP or MCP Inspector):

```
http://localhost:3100/mcp
```

## Usage Examples

### Start HTTP server

```bash
ZEALOT_URL=http://localhost:7377 ZEALOT_API_KEY=mykey \
  cargo run -p zealot-mcp -- --mode http --port 3100
```

### Health check

```bash
curl http://localhost:3100/health
# → ok
```

### MCP Inspector (interactive browser-based testing)

```bash
npx @modelcontextprotocol/inspector http://localhost:3100/mcp
```

### Stdio smoke test (no server running — just check tools/list response)

```bash
printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0"}}}\n{"jsonrpc":"2.0","method":"notifications/initialized"}\n{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}\n' | \
  ZEALOT_URL=http://localhost:7377 ZEALOT_API_KEY=mykey \
  cargo run -p zealot-mcp 2>/dev/null | tail -1 | jq '.result.tools | length'
```

Expected: `42` (14 wiki + 10 planner + 15 automation + 3 media tools)

## Verify

```bash
# Full build
cargo build -p zealot-mcp

# Type check only
cargo check -p zealot-mcp

# Start HTTP mode and hit health endpoint
cargo run -p zealot-mcp -- --mode http --port 3100 &
sleep 2
curl -s http://localhost:3100/health
kill %1
```
