use zealot_domain::account::{AccountDto, CreateApiKeyResponseDto, CreateApiKeyWithCredentialsDto};

use crate::{ApiError, ZealotClient};

/// Mint an API key from username + password (`POST /auth/api_key`).
/// This is the unauthenticated login path for headless clients — no session
/// or existing key needed. The raw key is returned exactly once.
pub async fn create_api_key_with_credentials(
    server_url: &str,
    username: &str,
    password: &str,
    label: &str,
) -> Result<CreateApiKeyResponseDto, ApiError> {
    let dto = CreateApiKeyWithCredentialsDto {
        username: username.to_string(),
        password: password.to_string(),
        label: Some(label.to_string()),
    };
    // An empty API key is fine here: the endpoint itself is credential-based.
    let client = ZealotClient::new(server_url, "");
    client.post("/auth/api_key", &dto).await
}

impl ZealotClient {
    /// `GET /auth/` — returns the authenticated account or 401.
    pub async fn whoami(&self) -> Result<AccountDto, ApiError> {
        self.get("/auth/").await
    }
}
