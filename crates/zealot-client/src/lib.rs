//! Shared HTTP client for Zealot terminal and machine clients (CLI, TUI, MCP).
//!
//! Provides the [`ZealotClient`] transport (X-API-Key auth over reqwest), typed
//! endpoint wrappers over the Zealot REST API, and config-file/profile handling
//! for storing server credentials.

pub mod api;
pub mod config;
pub mod error;
pub mod http;
pub mod types;

pub use config::{Config, Profile, ResolvedConnection};
pub use error::{ApiError, ConfigError};
pub use http::ZealotClient;
