//! The HTTP transport now lives in the shared `zealot-client` crate (also used
//! by the CLI and TUI apps). Re-exported here so existing `crate::client::…`
//! paths keep working.

pub use zealot_client::{ApiError, ZealotClient};
