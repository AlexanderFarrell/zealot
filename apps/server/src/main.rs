use std::sync::Arc;
use zealot_app::{
    app::AppState,
    config::ZealotConfig,
    ports::ZealotPorts,
};
use zealot_infra::{
    ports::{
        media::filesystem::MediaFilesystemPort,
        password::bcrypt_password::BcryptPasswordPort,
    },
    repos::get_repo_from_config,
};

#[tokio::main]
async fn main() -> Result<(), String> {
    let config = ZealotConfig::load_from_env();
    let repos = get_repo_from_config(&config)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    let ports = ZealotPorts {
        media: Arc::new(MediaFilesystemPort::new(&config)),
        password: Arc::new(BcryptPasswordPort::new()),
    };

    let state = AppState::new(repos, ports);

    zealot_api::http::run_http(state, config).await
}
