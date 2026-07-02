//! Config file + profile handling for terminal clients.
//!
//! Default location: `$XDG_CONFIG_HOME/zealot/config.toml` (falling back to
//! `~/.config/zealot/config.toml`), overridable with `$ZEALOT_CONFIG`.
//!
//! Connection resolution precedence (highest first):
//! 1. `ZEALOT_URL` / `ZEALOT_API_KEY` environment variables
//! 2. an explicitly selected profile (`--profile` flag or `ZEALOT_PROFILE`)
//! 3. the config file's `default_profile`

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::ConfigError;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_profile: Option<String>,
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Profile {
    pub server_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_id: Option<i64>,
    /// Title (or id) of the item that `zealot journal` comments on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub journal_item: Option<String>,
}

/// A fully resolved connection ready to build a `ZealotClient` from.
#[derive(Debug, Clone)]
pub struct ResolvedConnection {
    pub server_url: String,
    pub api_key: String,
    /// Name of the profile this came from, if it came from the config file.
    pub profile: Option<String>,
}

pub const DEFAULT_SERVER_URL: &str = "http://localhost:8456";

impl Config {
    /// Path of the config file, honoring `$ZEALOT_CONFIG` and `$XDG_CONFIG_HOME`.
    pub fn default_path() -> Result<PathBuf, ConfigError> {
        if let Ok(path) = std::env::var("ZEALOT_CONFIG") {
            if !path.is_empty() {
                return Ok(PathBuf::from(path));
            }
        }
        let base = match std::env::var("XDG_CONFIG_HOME") {
            Ok(dir) if !dir.is_empty() => PathBuf::from(dir),
            _ => {
                let home = std::env::var("HOME").map_err(|_| ConfigError::NoConfigDir)?;
                Path::new(&home).join(".config")
            }
        };
        Ok(base.join("zealot").join("config.toml"))
    }

    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        match std::fs::read_to_string(path) {
            Ok(text) => toml::from_str(&text).map_err(|source| ConfigError::Parse {
                path: path.display().to_string(),
                source,
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(source) => Err(ConfigError::Read {
                path: path.display().to_string(),
                source,
            }),
        }
    }

    /// Atomic write (tmp file + rename), 0600 on Unix since it holds the API key.
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        let write_err = |source| ConfigError::Write {
            path: path.display().to_string(),
            source,
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(write_err)?;
        }
        let text = toml::to_string_pretty(self)?;
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, &text).map_err(write_err)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600))
                .map_err(write_err)?;
        }
        std::fs::rename(&tmp, path).map_err(write_err)?;
        Ok(())
    }

    /// Name of the profile that would be used (explicit selection, then
    /// `$ZEALOT_PROFILE`, then `default_profile`, then sole profile).
    pub fn select_profile(&self, explicit: Option<&str>) -> Option<String> {
        if let Some(name) = explicit {
            return Some(name.to_string());
        }
        if let Ok(name) = std::env::var("ZEALOT_PROFILE") {
            if !name.is_empty() {
                return Some(name);
            }
        }
        if let Some(name) = &self.default_profile {
            return Some(name.clone());
        }
        if self.profiles.len() == 1 {
            return self.profiles.keys().next().cloned();
        }
        None
    }

    pub fn profile(&self, explicit: Option<&str>) -> Result<(String, &Profile), ConfigError> {
        let name = self.select_profile(explicit).ok_or(ConfigError::NotLoggedIn)?;
        let profile = self
            .profiles
            .get(&name)
            .ok_or_else(|| ConfigError::NoSuchProfile(name.clone()))?;
        Ok((name, profile))
    }

    /// Resolve a connection, applying env-var overrides on top of the config.
    pub fn resolve(&self, explicit_profile: Option<&str>) -> Result<ResolvedConnection, ConfigError> {
        let env_url = std::env::var("ZEALOT_URL").ok().filter(|s| !s.is_empty());
        let env_key = std::env::var("ZEALOT_API_KEY").ok().filter(|s| !s.is_empty());

        // Full env override: no config file needed at all (CI/scripting).
        if let (Some(url), Some(key)) = (env_url.clone(), env_key.clone()) {
            return Ok(ResolvedConnection {
                server_url: url,
                api_key: key,
                profile: None,
            });
        }

        let (name, profile) = self.profile(explicit_profile)?;
        let api_key = env_key
            .or_else(|| profile.api_key.clone())
            .ok_or(ConfigError::NotLoggedIn)?;
        Ok(ResolvedConnection {
            server_url: env_url.unwrap_or_else(|| profile.server_url.clone()),
            api_key,
            profile: Some(name),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Config {
        let mut config = Config::default();
        config.default_profile = Some("home".into());
        config.profiles.insert(
            "home".into(),
            Profile {
                server_url: "http://localhost:8456".into(),
                api_key: Some("secret".into()),
                api_key_id: Some(7),
                journal_item: Some("Journal".into()),
            },
        );
        config
    }

    #[test]
    fn round_trips_through_toml() {
        let config = sample();
        let text = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&text).unwrap();
        assert_eq!(parsed.default_profile.as_deref(), Some("home"));
        let p = &parsed.profiles["home"];
        assert_eq!(p.server_url, "http://localhost:8456");
        assert_eq!(p.api_key.as_deref(), Some("secret"));
        assert_eq!(p.api_key_id, Some(7));
        assert_eq!(p.journal_item.as_deref(), Some("Journal"));
    }

    #[test]
    fn missing_file_loads_default() {
        let config = Config::load(Path::new("/nonexistent/zealot-config.toml")).unwrap();
        assert!(config.profiles.is_empty());
    }

    #[test]
    fn single_profile_is_selected_without_default() {
        let mut config = sample();
        config.default_profile = None;
        assert_eq!(config.select_profile(None).as_deref(), Some("home"));
    }

    #[test]
    fn explicit_profile_wins() {
        let config = sample();
        assert_eq!(config.select_profile(Some("work")).as_deref(), Some("work"));
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        sample().save(&path).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
        let loaded = Config::load(&path).unwrap();
        assert_eq!(loaded.profiles["home"].api_key.as_deref(), Some("secret"));
    }
}
