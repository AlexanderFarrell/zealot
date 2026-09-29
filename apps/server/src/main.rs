mod backup;

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
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
        broadcast_event_port::BroadcastEventPort, media::filesystem::MediaFilesystemPort,
        password::bcrypt_password::BcryptPasswordPort,
    },
    repos::get_repo_from_config,
};
use zealot_lua::runner::LuaRuleRunner;

#[derive(Parser)]
#[command(
    name = "zealot-server",
    about = "Zealot server and operator recovery commands"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}
#[derive(Subcommand)]
enum Command {
    Backup(Backup),
    Restore(Restore),
}
#[derive(Args)]
struct Backup {
    #[command(subcommand)]
    command: BackupCommand,
}
#[derive(Subcommand)]
enum BackupCommand {
    Create {
        #[arg(long)]
        destination: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    Verify {
        bundle: PathBuf,
        #[arg(long)]
        json: bool,
    },
    Status {
        #[arg(long)]
        destination: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
}
#[derive(Args)]
struct Restore {
    bundle: PathBuf,
    #[arg(long)]
    force: bool,
    #[arg(long)]
    json: bool,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let config = ZealotConfig::load_from_env();
    if let Some(command) = cli.command {
        return operator(command, &config);
    }
    // This happens before get_repo_from_config, whose successful connection applies migrations.
    if config.backup_enabled && source_exists(&config) {
        backup::create(&config, None, "pre-start")
            .map_err(|e| format!("pre-start backup failed; migrations were not run: {e}"))?;
    }
    let repos = get_repo_from_config(&config)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    // Build services once with a noop runner to resolve the circular dependency
    // (LuaRuleRunner needs ZealotServices; ZealotServices needs a rule_runner port).
    let bootstrap_ports = ZealotPorts {
        media: Arc::new(MediaFilesystemPort::new(&config)),
        password: Arc::new(BcryptPasswordPort::new()),
        events: Arc::new(NoopEventPort),
        rule_runner: Arc::new(NoopRuleRunner),
    };
    let services = Arc::new(ZealotServices::new(bootstrap_ports.clone(), repos.clone()));

    let (tx, mut rx) = tokio::sync::broadcast::channel::<ZealotEvent>(256);

    let ports = ZealotPorts {
        events: Arc::new(BroadcastEventPort::new(tx)),
        rule_runner: Arc::new(LuaRuleRunner::new(Arc::new(repos.clone()), services)),
        ..bootstrap_ports
    };

    let state = AppState::new(repos, ports);

    zealot_app::scheduler::start(state.clone());
    if config.backup_enabled {
        start_backup_schedule(config.clone());
    }

    let event_runner = state.ports.rule_runner.clone();
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    event_runner.run_event_rules(event).await;
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("Event loop lagged, skipped {n} events");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    zealot_api::http::run_http(state, config).await
}

fn operator(command: Command, config: &ZealotConfig) -> Result<(), String> {
    match command {
        Command::Backup(Backup {
            command: BackupCommand::Create { destination, json },
        }) => {
            let bundle = backup::create(config, destination.as_deref(), "manual")?;
            output(json, serde_json::json!({"bundle":bundle,"verified":true}));
        }
        Command::Backup(Backup {
            command: BackupCommand::Verify { bundle, json },
        }) => {
            backup::verify(config, &bundle)?;
            output(json, serde_json::json!({"bundle":bundle,"verified":true}));
        }
        Command::Backup(Backup {
            command: BackupCommand::Status { destination, json },
        }) => {
            let status = backup::status(config, destination.as_deref())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&status).map_err(|e| e.to_string())?
                );
            } else {
                println!("{} backup bundle(s)", status.bundles.len());
                if let Some(last) = status.latest {
                    println!("latest: {} ({})", last.outcome, last.time)
                }
            }
        }
        Command::Restore(r) => {
            backup::restore(config, &r.bundle, r.force)?;
            output(
                r.json,
                serde_json::json!({"bundle":r.bundle,"restored":true}),
            );
        }
    };
    Ok(())
}
fn output(json: bool, value: serde_json::Value) {
    if json {
        println!("{}", value)
    } else {
        println!("{}", value)
    }
}
fn source_exists(c: &ZealotConfig) -> bool {
    c.database == "postgres"
        || (c.database == "sqlite" && std::path::Path::new(&c.db_filename).exists())
}
fn start_backup_schedule(config: ZealotConfig) {
    use croner::Cron;
    let Ok(cron) = Cron::new(&config.backup_schedule).parse() else {
        tracing::error!(schedule=%config.backup_schedule,"invalid BACKUP_SCHEDULE; scheduled backups disabled");
        return;
    };
    tokio::spawn(async move {
        let mut timer = tokio::time::interval(std::time::Duration::from_secs(30));
        let mut last_minute = None;
        loop {
            timer.tick().await;
            let now = chrono::Utc::now();
            let minute = now.format("%Y%m%d%H%M").to_string();
            if last_minute.as_ref() == Some(&minute)
                || !cron.is_time_matching(&now).unwrap_or(false)
            {
                continue;
            }
            last_minute = Some(minute);
            let c = config.clone();
            match tokio::task::spawn_blocking(move || backup::create(&c, None, "scheduled")).await {
                Ok(Ok(_)) => tracing::info!("scheduled backup completed"),
                Ok(Err(e)) => tracing::error!(error=%e,"scheduled backup failed"),
                Err(e) => tracing::error!(error=%e,"scheduled backup task failed"),
            }
        }
    });
}
