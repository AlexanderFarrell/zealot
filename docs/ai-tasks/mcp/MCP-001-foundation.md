# MCP-001: Foundation

## Goal

Establish the skeleton for `apps/mcp/` — dependencies, configuration, HTTP client, transport dispatch, and stub implementations of the server handler, tool router, and prompt router. All subsequent tickets layer on top of this.

## Background

The MCP server is a standalone binary that acts as a bridge:
- **Left side**: speaks the Model Context Protocol (JSON-RPC 2.0) over stdio or HTTP+SSE
- **Right side**: makes authenticated HTTP calls to the Zealot API using `X-API-Key`

It uses the `rmcp` crate (official Rust MCP SDK from the modelcontextprotocol org) for protocol handling.

## Files to Create / Modify

### `apps/mcp/Cargo.toml`

Replace the stub with:

```toml
[package]
name = "zealot-mcp"
version.workspace = true
edition.workspace = true

[dependencies]
# MCP protocol
rmcp = { version = "1", features = [
    "server",
    "macros",
    "transport-io",
    "transport-streamable-http-server",
    "schemars",
] }
schemars = "1"

# HTTP client for Zealot API
reqwest = { version = "0.12", features = ["json"] }

# Web framework (HTTP mode)
axum.workspace = true
tower-http.workspace = true

# Async runtime
tokio.workspace = true

# Serialization
serde.workspace = true
serde_json.workspace = true

# CLI argument parsing
clap = { version = "4", features = ["derive", "env"] }

# Error handling + logging
anyhow.workspace = true
thiserror.workspace = true
tracing.workspace = true
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
```

### `apps/mcp/src/main.rs`

```rust
mod config;
mod client;
mod server;
mod tools;
mod prompts;

use clap::Parser;
use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // CRITICAL: log to stderr so stdout stays clean for stdio MCP transport
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let config = Config::parse();
    config.validate()?;

    match config.mode {
        config::Mode::Stdio => server::run_stdio(config).await,
        config::Mode::Http => server::run_http(config).await,
    }
}
```

### `apps/mcp/src/config.rs`

```rust
use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Parser)]
#[command(name = "zealot-mcp", about = "Zealot MCP server — connect Claude to your Zealot wiki and planner")]
pub struct Config {
    /// Base URL of the Zealot API server
    #[arg(long, env = "ZEALOT_URL", default_value = "http://localhost:7377")]
    pub url: String,

    /// API key for Zealot authentication (X-API-Key header)
    #[arg(long, env = "ZEALOT_API_KEY")]
    pub api_key: String,

    /// Transport mode: stdio (for Claude Desktop) or http (for web clients)
    #[arg(long, default_value = "stdio")]
    pub mode: Mode,

    /// Port for HTTP mode
    #[arg(long, env = "MCP_PORT", default_value = "3100")]
    pub port: u16,
}

impl Config {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.api_key.is_empty() {
            anyhow::bail!("ZEALOT_API_KEY or --api-key must be provided");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Mode {
    Stdio,
    Http,
}
```

### `apps/mcp/src/client.rs`

The Zealot HTTP client. All Zealot API calls go through this struct.

```rust
use reqwest::{Client, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("not found")]
    NotFound,
    #[error("HTTP {status}: {message}")]
    Http { status: StatusCode, message: String },
    #[error(transparent)]
    Request(#[from] reqwest::Error),
}

#[derive(Clone)]
pub struct ZealotClient {
    inner: Client,
    base_url: String,
    api_key: String,
}

impl ZealotClient {
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            inner: Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key: api_key.into(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let resp = self.inner.get(self.url(path))
            .header("X-API-Key", &self.api_key)
            .send().await?;
        self.parse(resp).await
    }

    pub async fn post<B: Serialize, T: DeserializeOwned>(&self, path: &str, body: &B) -> Result<T, ApiError> {
        let resp = self.inner.post(self.url(path))
            .header("X-API-Key", &self.api_key)
            .json(body)
            .send().await?;
        self.parse(resp).await
    }

    pub async fn patch<B: Serialize, T: DeserializeOwned>(&self, path: &str, body: &B) -> Result<T, ApiError> {
        let resp = self.inner.patch(self.url(path))
            .header("X-API-Key", &self.api_key)
            .json(body)
            .send().await?;
        self.parse(resp).await
    }

    pub async fn put<B: Serialize, T: DeserializeOwned>(&self, path: &str, body: &B) -> Result<T, ApiError> {
        let resp = self.inner.put(self.url(path))
            .header("X-API-Key", &self.api_key)
            .json(body)
            .send().await?;
        self.parse(resp).await
    }

    pub async fn delete(&self, path: &str) -> Result<(), ApiError> {
        let resp = self.inner.delete(self.url(path))
            .header("X-API-Key", &self.api_key)
            .send().await?;
        if resp.status() == StatusCode::NOT_FOUND { return Err(ApiError::NotFound); }
        if !resp.status().is_success() {
            let status = resp.status();
            let message = resp.text().await.unwrap_or_default();
            return Err(ApiError::Http { status, message });
        }
        Ok(())
    }

    async fn parse<T: DeserializeOwned>(&self, resp: reqwest::Response) -> Result<T, ApiError> {
        if resp.status() == StatusCode::NOT_FOUND { return Err(ApiError::NotFound); }
        if !resp.status().is_success() {
            let status = resp.status();
            let message = resp.text().await.unwrap_or_default();
            return Err(ApiError::Http { status, message });
        }
        Ok(resp.json::<T>().await?)
    }
}
```

### `apps/mcp/src/tools/mod.rs` (stub — expanded in MCP-002 through MCP-005)

```rust
pub mod wiki;
pub mod planner;
pub mod automation;
pub mod media;

use rmcp::{ServerHandler, model::{ServerInfo, ServerCapabilities, Implementation, ProtocolVersion}};
use crate::{client::ZealotClient, config::Config};

#[derive(Clone)]
pub struct ZealotServer {
    pub client: ZealotClient,
}

impl ZealotServer {
    pub fn new(config: &Config) -> Self {
        Self {
            client: ZealotClient::new(&config.url, &config.api_key),
        }
    }
}

impl ServerHandler for ZealotServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .build(),
            server_info: Implementation::new("zealot-mcp", env!("CARGO_PKG_VERSION")),
            instructions: Some(
                "Zealot personal wiki and planner. Use the tools to read and write wiki items, \
                manage your planner and habits, run automations, and interact with your knowledge base. \
                Items have types, attributes, relationships, and content written in ZealotScript (markdown-like). \
                Dates are always YYYY-MM-DD format.".to_string()
            ),
        }
    }
}
```

### `apps/mcp/src/prompts/mod.rs` (stub — expanded in MCP-006)

```rust
// Prompts implemented in MCP-006
```

### `apps/mcp/src/server.rs` (stub — completed in MCP-007)

```rust
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use crate::{config::Config, tools::ZealotServer};

pub async fn run_stdio(config: Config) -> anyhow::Result<()> {
    tracing::info!("Starting Zealot MCP server (stdio)");
    let service = ZealotServer::new(&config);
    let server = service.serve(stdio()).await
        .map_err(|e| anyhow::anyhow!("MCP serve error: {e}"))?;
    server.waiting().await
        .map_err(|e| anyhow::anyhow!("MCP wait error: {e}"))?;
    Ok(())
}

pub async fn run_http(config: Config) -> anyhow::Result<()> {
    // Completed in MCP-007
    todo!("HTTP transport — see MCP-007")
}
```

## Verify

```bash
cargo check -p zealot-mcp
```
