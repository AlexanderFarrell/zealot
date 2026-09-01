pub mod auth;
pub mod comment;
pub mod item;
pub mod media;
pub mod meta;
pub mod planner;
pub mod rule;
pub mod statistic;

use anyhow::Result;

use crate::cli::{Cli, Command};
use crate::context::Ctx;

pub async fn run(cli: Cli) -> Result<()> {
    // Commands that work without a server connection.
    match &cli.command {
        Command::Completions { shell } => {
            use clap::CommandFactory;
            clap_complete::generate(
                *shell,
                &mut Cli::command(),
                "zealot",
                &mut std::io::stdout(),
            );
            return Ok(());
        }
        Command::Login { url, name } => {
            return auth::login(cli.profile.as_deref(), url.as_deref(), name).await;
        }
        Command::Tui => return launch_tui(),
        _ => {}
    }

    let ctx = Ctx::connect(
        cli.profile.as_deref(),
        cli.url.as_deref(),
        cli.api_key.as_deref(),
        cli.json,
    )?;

    match cli.command {
        Command::Login { .. } | Command::Completions { .. } | Command::Tui => unreachable!(),
        Command::Logout { keep_key } => auth::logout(ctx, keep_key).await,
        Command::Status => auth::status(ctx).await,

        Command::Item(cmd) => item::run(ctx, cmd).await,
        Command::View(args) => item::view(&ctx, args).await,
        Command::Add(args) => item::new_item(&ctx, args).await,
        Command::Search {
            term,
            content,
            heading,
            regex,
            limit,
            offset,
            all,
        } => item::search(&ctx, &term, content, heading, regex, limit, offset, all).await,
        Command::Filter {
            exprs,
            limit,
            offset,
        } => item::filter(&ctx, &exprs, limit, offset).await,

        Command::Day { date, full } => planner::day(&ctx, date.unwrap_or_default(), full).await,
        Command::Week { week } => planner::week(&ctx, week).await,
        Command::Month { month, year } => planner::month(&ctx, month, year).await,
        Command::Year { year } => planner::year(&ctx, year).await,
        Command::Habit(cmd) => planner::habit(&ctx, cmd).await,
        Command::Block(cmd) => planner::block(&ctx, cmd).await,

        Command::Comment(cmd) => comment::run(&ctx, cmd).await,
        Command::Journal { text } => comment::journal(&ctx, &text).await,

        Command::ItemType(cmd) => meta::item_type(&ctx, cmd).await,
        Command::Attr(cmd) => meta::attr_kind(&ctx, cmd).await,
        Command::Rule(cmd) => rule::run(&ctx, cmd).await,
        Command::Statistic(cmd) => statistic::run(&ctx, cmd).await,
        Command::Media(cmd) => media::run(&ctx, cmd).await,

        Command::Api { method, path, body } => api(&ctx, &method, &path, body.as_deref()).await,
    }
}

/// Exec `zealot-tui`, looking next to our own binary first, then on PATH.
fn launch_tui() -> Result<()> {
    let sibling = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("zealot-tui")))
        .filter(|p| p.exists());
    let program = sibling.unwrap_or_else(|| "zealot-tui".into());
    let status = std::process::Command::new(&program)
        .status()
        .map_err(|e| anyhow::anyhow!("failed to launch {}: {e}", program.display()))?;
    if !status.success() {
        anyhow::bail!("zealot-tui exited with {status}");
    }
    Ok(())
}

/// Raw API escape hatch.
async fn api(ctx: &Ctx, method: &str, path: &str, body: Option<&str>) -> Result<()> {
    use std::str::FromStr;
    let method = zealot_client::Method::from_str(&method.to_uppercase())
        .map_err(|_| anyhow::anyhow!("invalid HTTP method '{method}'"))?;
    let body = match body {
        Some("-") => Some(serde_json::from_str(&crate::context::read_stdin()?)?),
        Some(text) => Some(serde_json::from_str(text)?),
        None => None,
    };
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    let (status, text) = ctx.client.raw(method, &path, body).await?;
    // Pretty-print JSON responses; pass through anything else.
    match serde_json::from_str::<serde_json::Value>(&text) {
        Ok(value) => println!("{}", serde_json::to_string_pretty(&value)?),
        Err(_) => println!("{text}"),
    }
    if !status.is_success() {
        anyhow::bail!("HTTP {status}");
    }
    Ok(())
}
