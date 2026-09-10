use std::sync::Arc;

use crate::{repos::scope::{ScopeRepo, ServerMetadata}, repos::common::RepoError};

#[derive(Debug, Clone)]
pub struct ScopeService {
    repo: Arc<dyn ScopeRepo>,
}

impl ScopeService {
    pub fn new(repo: &Arc<dyn ScopeRepo>) -> Self { Self { repo: repo.clone() } }

    pub fn server_metadata(&self) -> Result<ServerMetadata, RepoError> {
        self.repo.server_metadata()
    }
}
