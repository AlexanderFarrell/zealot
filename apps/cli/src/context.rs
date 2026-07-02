use std::path::PathBuf;

use anyhow::{Context as _, Result};
use zealot_client::{Config, ZealotClient};
use zealot_domain::item::ItemDto;

use crate::args::ItemRef;

/// Everything a command handler needs: the connected client, the loaded
/// config, and output preferences.
pub struct Ctx {
    pub client: ZealotClient,
    pub config: Config,
    pub config_path: PathBuf,
    pub profile: Option<String>,
    pub json: bool,
}

impl Ctx {
    /// Build a context for commands that talk to the server.
    pub fn connect(
        profile: Option<&str>,
        url_override: Option<&str>,
        key_override: Option<&str>,
        json: bool,
    ) -> Result<Self> {
        let config_path = Config::default_path()?;
        let config = Config::load(&config_path)?;

        // CLI flags behave like the env overrides: either one wins over config.
        let (server_url, api_key, profile_name) = match (url_override, key_override) {
            (Some(url), Some(key)) => (url.to_string(), key.to_string(), None),
            _ => {
                let resolved = config.resolve(profile)?;
                (
                    url_override
                        .map(String::from)
                        .unwrap_or(resolved.server_url),
                    key_override.map(String::from).unwrap_or(resolved.api_key),
                    resolved.profile,
                )
            }
        };

        Ok(Self {
            client: ZealotClient::new(server_url, api_key),
            config,
            config_path,
            profile: profile_name,
            json,
        })
    }

    /// Resolve an item reference (id or title) to the full item.
    pub async fn resolve_item(&self, item: &ItemRef) -> Result<ItemDto> {
        match item {
            ItemRef::Id(id) => self
                .client
                .get_item(*id)
                .await
                .with_context(|| format!("item #{id} not found")),
            ItemRef::Title(title) => self
                .client
                .get_item_by_title(title)
                .await
                .with_context(|| format!("no item titled '{title}'")),
        }
    }

    /// The profile currently selected in the config, for mutation (login/logout).
    pub fn profile_entry(&mut self) -> Option<(String, &mut zealot_client::Profile)> {
        let name = self.config.select_profile(self.profile.as_deref())?;
        let profile = self.config.profiles.get_mut(&name)?;
        Some((name, profile))
    }
}

/// Read all of stdin (for piped content).
pub fn read_stdin() -> Result<String> {
    use std::io::Read;
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .context("failed to read stdin")?;
    Ok(buf)
}

/// Join positional text args, falling back to stdin when empty or "-".
pub fn text_or_stdin(words: &[String]) -> Result<String> {
    let joined = words.join(" ");
    if joined.is_empty() || joined == "-" {
        Ok(read_stdin()?.trim_end().to_string())
    } else {
        Ok(joined)
    }
}

/// Hostname for API key labels, best-effort.
pub fn hostname() -> String {
    if let Ok(name) = std::env::var("HOSTNAME") {
        if !name.is_empty() {
            return name;
        }
    }
    std::fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown-host".to_string())
}
