use sqlx::PgPool;
use std::sync::Arc;
use zealot_api::http;
use zealot_app::{
    app::AppState,
    config::ZealotConfig,
    ports::ZealotPorts,
    repos::ZealotRepos,
    services::{
        ZealotServices,
        account::AccountService,
        attribute::{self, AttributeService},
    },
};
use zealot_infra::{
    ports::media::filesystem::MediaFilesystemPort,
    repos::{
        get_repo_from_config,
        postgres::{
            account_postgres::AccountPostgresRepo, attribute_postgres::AttributePostgresRepo,
            get_postgres_repos,
        },
    },
};

#[tokio::main]
async fn main() -> Result<(), String> {
    let config = ZealotConfig::load_from_env();
    let repos = get_repo_from_config(&config)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    let ports = ZealotPorts {
        media: Arc::new(MediaFilesystemPort::new(&config)),
    };

    let state = AppState {
        services: ZealotServices {
            account: Arc::new(AccountService::new(&repos.account)),
            attribute: Arc::new(AttributeService::new(&repos.attribute)),
            repos,
            ports,
        },
    };

    zealot_api::http::run_http(state, config).await
}
