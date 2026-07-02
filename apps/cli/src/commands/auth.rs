use anyhow::{Context, Result, bail};
use zealot_client::{Config, Profile, api::auth::create_api_key_with_credentials};

use crate::context::{Ctx, hostname};
use crate::output::{bold, dim, green, print_json, red};

/// Interactive login: prompt for credentials, mint an API key, store it in the
/// config file. Mirrors the desktop app's credential flow.
pub async fn login(profile_flag: Option<&str>, url: Option<&str>, name: &str) -> Result<()> {
    let config_path = Config::default_path()?;
    let mut config = Config::load(&config_path)?;
    let profile_name = profile_flag.unwrap_or(name).to_string();

    let default_url = config
        .profiles
        .get(&profile_name)
        .map(|p| p.server_url.clone())
        .unwrap_or_else(|| zealot_client::config::DEFAULT_SERVER_URL.to_string());

    let server_url = match url {
        Some(url) => url.to_string(),
        None => {
            let input = prompt(&format!("Server URL [{default_url}]: "))?;
            if input.is_empty() { default_url } else { input }
        }
    };
    let server_url = server_url.trim_end_matches('/').to_string();

    let username = prompt("Username: ")?;
    if username.is_empty() {
        bail!("username is required");
    }
    let password =
        rpassword::prompt_password("Password: ").context("failed to read password")?;

    let label = format!("CLI ({})", hostname());
    let resp = create_api_key_with_credentials(&server_url, &username, &password, &label)
        .await
        .context("login failed")?;

    let entry = config
        .profiles
        .entry(profile_name.clone())
        .or_insert_with(Profile::default);
    entry.server_url = server_url.clone();
    entry.api_key = Some(resp.key);
    entry.api_key_id = Some(resp.api_key_id);
    if config.default_profile.is_none() {
        config.default_profile = Some(profile_name.clone());
    }
    config.save(&config_path)?;

    println!(
        "{} Logged in to {} as {} (profile '{}', key '{}')",
        green("✔"),
        bold(&server_url),
        bold(&username),
        profile_name,
        resp.label
    );
    println!("{}", dim(&format!("Credentials stored in {}", config_path.display())));
    Ok(())
}

pub async fn logout(mut ctx: Ctx, keep_key: bool) -> Result<()> {
    let Some((name, profile)) = ctx.profile_entry() else {
        bail!("no profile to log out of");
    };

    if !keep_key {
        if let Some(key_id) = profile.api_key_id {
            match ctx.client.revoke_api_key(key_id).await {
                Ok(()) => println!("{} Revoked API key #{key_id}", green("✔")),
                Err(e) => println!(
                    "{} Could not revoke key #{key_id} server-side: {e}",
                    red("✘")
                ),
            }
        }
    }

    // Re-borrow after the await (profile_entry borrows ctx.config mutably).
    if let Some(profile) = ctx.config.profiles.get_mut(&name) {
        profile.api_key = None;
        profile.api_key_id = None;
    }
    ctx.config.save(&ctx.config_path)?;
    println!("Logged out of profile '{name}'.");
    Ok(())
}

pub async fn status(ctx: Ctx) -> Result<()> {
    let account = ctx.client.whoami().await;

    if ctx.json {
        let status = serde_json::json!({
            "server_url": ctx.client.base_url(),
            "profile": ctx.profile,
            "config_path": ctx.config_path.display().to_string(),
            "authenticated": account.is_ok(),
            "account": account.as_ref().ok(),
        });
        return print_json(&status);
    }

    println!("{}  {}", bold("Server"), ctx.client.base_url());
    if let Some(profile) = &ctx.profile {
        println!("{} {}", bold("Profile"), profile);
    }
    println!("{}  {}", bold("Config"), ctx.config_path.display());
    match account {
        Ok(account) => {
            println!(
                "{} {} ({} {})",
                bold("Account"),
                green(&account.username),
                account.given_name,
                account.surname
            );
            Ok(())
        }
        Err(e) => {
            println!("{} {}", bold("Account"), red("not authenticated"));
            Err(e.into())
        }
    }
}

fn prompt(message: &str) -> Result<String> {
    use std::io::Write;
    print!("{message}");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
