use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("not found")]
    NotFound,
    #[error("HTTP {status}: {message}")]
    Http { status: StatusCode, message: String },
    #[error(transparent)]
    Request(#[from] reqwest::Error),
}

impl ApiError {
    pub fn is_unauthorized(&self) -> bool {
        matches!(
            self,
            ApiError::Http {
                status: StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN,
                ..
            }
        )
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("could not determine config directory (set HOME or ZEALOT_CONFIG)")]
    NoConfigDir,
    #[error("failed to read config {path}: {source}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to write config {path}: {source}")]
    Write {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse config {path}: {source}")]
    Parse {
        path: String,
        source: toml::de::Error,
    },
    #[error("failed to serialize config: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("no profile named '{0}' in config (run `zealot login`)")]
    NoSuchProfile(String),
    #[error("not logged in (run `zealot login`)")]
    NotLoggedIn,
}
