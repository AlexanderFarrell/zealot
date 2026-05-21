use clap::Parser;
use zealot_mcp::config::{Config, Mode};
use zealot_mcp::server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // CRITICAL: log to stderr so stdout stays clean for the stdio MCP transport
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let config = Config::parse();
    config.validate()?;

    match config.mode {
        Mode::Stdio => server::run_stdio(config).await,
        Mode::Http  => server::run_http(config).await,
    }
}
