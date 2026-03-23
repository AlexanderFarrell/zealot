use crate::services::ZealotServices;

#[derive(Debug, Clone)]
pub struct AppState {
    pub services: ZealotServices,
}
