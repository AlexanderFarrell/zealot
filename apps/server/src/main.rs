use std::sync::Arc;
use zealot_app::{
    app::AppState,
    config::ZealotConfig,
    ports::{
        ZealotPorts,
        events::NoopEventPort,
        rule_runner::NoopRuleRunner,
    },
    services::ZealotServices,
};
use zealot_infra::{
    ports::{
        media::filesystem::MediaFilesystemPort,
        password::bcrypt_password::BcryptPasswordPort,
    },
    repos::get_repo_from_config,
};
use zealot_lua::runner::LuaRuleRunner;

#[tokio::main]
async fn main() -> Result<(), String> {
    let config = ZealotConfig::load_from_env();
    let repos = get_repo_from_config(&config)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    // Build services once with a noop runner to resolve the circular dependency
    // (LuaRuleRunner needs ZealotServices; ZealotServices needs a rule_runner port).
    let bootstrap_ports = ZealotPorts {
        media:       Arc::new(MediaFilesystemPort::new(&config)),
        password:    Arc::new(BcryptPasswordPort::new()),
        events:      Arc::new(NoopEventPort),
        rule_runner: Arc::new(NoopRuleRunner),
    };
    let services = Arc::new(ZealotServices::new(bootstrap_ports.clone(), repos.clone()));

    let ports = ZealotPorts {
        rule_runner: Arc::new(LuaRuleRunner::new(Arc::new(repos.clone()), services)),
        ..bootstrap_ports
    };

    let state = AppState::new(repos, ports);

    zealot_app::scheduler::start(state.clone());

    zealot_api::http::run_http(state, config).await
}
