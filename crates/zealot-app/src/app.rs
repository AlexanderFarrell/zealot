use crate::{ports::ZealotPorts, repos::ZealotRepos, services::ZealotServices};

pub struct AppState {
    pub repos: ZealotRepos,
    pub services: ZealotServices,
    pub ports: ZealotPorts,
}