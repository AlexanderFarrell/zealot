use std::path::PathBuf;

use anyhow::{Context as _, Result};
use sha2::{Digest, Sha256};
use zealot_client::{Config, ZealotClient};
use zealot_domain::item::ItemDto;

use crate::args::ItemRef;

/// Everything a command handler needs: the connected client, the loaded
/// config, and output preferences.
pub struct Ctx {
    pub client: ZealotClient,
    /// Stable, credential-safe namespace for durable local editor drafts.
    pub draft_identity: String,
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

        let draft_identity = draft_identity(&server_url, &api_key);

        Ok(Self {
            client: ZealotClient::new(server_url, api_key),
            draft_identity,
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

pub(crate) fn draft_identity(server_url: &str, api_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(server_url.trim_end_matches('/').as_bytes());
    hasher.update([0]);
    hasher.update(api_key.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::draft_identity;

    #[test]
    fn draft_identity_is_stable_and_does_not_expose_credentials() {
        let identity = draft_identity("https://zealot.example/", "top-secret-key");
        assert_eq!(
            identity,
            draft_identity("https://zealot.example", "top-secret-key")
        );
        assert_ne!(
            identity,
            draft_identity("https://zealot.example", "another-key")
        );
        assert!(!identity.contains("top-secret-key"));
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
