mod args;
mod cli;
mod commands;
mod context;
mod editor;
mod frontmatter;
mod output;

use clap::Parser;
use zealot_client::{ApiError, ConfigError};

/// Exit codes: 0 ok, 1 error, 2 usage (clap), 3 not found, 4 auth.
fn classify_error(err: &anyhow::Error) -> i32 {
    for cause in err.chain() {
        if let Some(api) = cause.downcast_ref::<ApiError>() {
            if matches!(api, ApiError::NotFound) {
                return 3;
            }
            if api.is_unauthorized() {
                return 4;
            }
        }
        if let Some(config) = cause.downcast_ref::<ConfigError>() {
            if matches!(
                config,
                ConfigError::NotLoggedIn | ConfigError::NoSuchProfile(_)
            ) {
                return 4;
            }
        }
    }
    1
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Die quietly on closed pipes (`zealot … | head`) instead of panicking.
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    let cli = cli::Cli::parse();

    // Color on only for interactive stdout, unless disabled explicitly.
    use std::io::IsTerminal;
    let color = !cli.no_color
        && std::env::var_os("NO_COLOR").is_none()
        && std::io::stdout().is_terminal()
        && !cli.json;
    output::set_color_enabled(color);

    match commands::run(cli).await {
        Ok(()) => {}
        Err(err) => {
            let code = classify_error(&err);
            eprintln!("{} {err:#}", output::red("error:"));
            if code == 4 {
                eprintln!("{}", output::dim("hint: run `zealot login` to authenticate"));
            }
            std::process::exit(code);
        }
    }
}
