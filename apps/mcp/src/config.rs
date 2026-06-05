use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Parser)]
#[command(
    name = "zealot-mcp",
    about = "Zealot MCP server — connect Claude to your Zealot wiki and planner"
)]
pub struct Config {
    /// Base URL of the Zealot API server
    #[arg(long, env = "ZEALOT_URL", default_value = "http://localhost:7377")]
    pub url: String,

    /// API key for Zealot authentication (X-API-Key header)
    #[arg(long, env = "ZEALOT_API_KEY")]
    pub api_key: String,

    /// Transport mode: stdio (for Claude Desktop) or http (for web clients)
    #[arg(long, env = "MCP_MODE", default_value = "stdio")]
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
