use std::sync::Arc;
use zealot_app::{
    app::AppState,
    config::ZealotConfig,
    ports::{
        ZealotPorts,
        events::{NoopEventPort, ZealotEvent},
        rule_runner::NoopRuleRunner,
    },
    services::ZealotServices,
};
use zealot_infra::{
    ports::{
        broadcast_event_port::BroadcastEventPort,
        media::filesystem::MediaFilesystemPort,
        password::bcrypt_password::BcryptPasswordPort,
    },
    repos::get_repo_from_config,
};
use zealot_lua::runner::LuaRuleRunner;

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

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

    let (tx, mut rx) = tokio::sync::broadcast::channel::<ZealotEvent>(256);

    let ports = ZealotPorts {
        events:      Arc::new(BroadcastEventPort::new(tx)),
        rule_runner: Arc::new(LuaRuleRunner::new(Arc::new(repos.clone()), services)),
        ..bootstrap_ports
    };

    let state = AppState::new(repos, ports);

    zealot_app::scheduler::start(state.clone());

    let event_runner = state.ports.rule_runner.clone();
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => { event_runner.run_event_rules(event).await; }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("Event loop lagged, skipped {n} events");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    zealot_api::http::run_http(state, config).await
}
