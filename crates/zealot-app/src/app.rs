use crate::{ports::ZealotPorts, repos::ZealotRepos, services::ZealotServices};

#[derive(Debug, Clone)]
pub struct AppState {
    pub services: ZealotServices,
}

impl AppState {
    pub fn new(repos: ZealotRepos, ports: ZealotPorts) -> Self {
        Self {
            services: ZealotServices::new(ports, repos)
        }
    }
}