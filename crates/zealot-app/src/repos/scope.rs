use std::fmt::Debug;

use crate::repos::common::RepoError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerMetadata {
    pub server_id: String,
    /// Latest SQL migration applied to this server. Future sync handshakes must
    /// require an exact match before exchanging scope data.
    pub schema_migration_version: i64,
}

pub trait ScopeRepo: Debug + Send + Sync {
    fn server_metadata(&self) -> Result<ServerMetadata, RepoError>;
}
