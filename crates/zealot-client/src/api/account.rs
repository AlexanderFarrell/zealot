use serde_json::json;
use zealot_domain::account::{ApiKeyRecordDto, CreateApiKeyResponseDto};

use crate::{ApiError, ZealotClient};

impl ZealotClient {
    pub async fn list_api_keys(&self) -> Result<Vec<ApiKeyRecordDto>, ApiError> {
        self.get("/account/api-keys").await
    }

    pub async fn create_api_key(&self, label: &str) -> Result<CreateApiKeyResponseDto, ApiError> {
        self.post("/account/api-keys", &json!({ "label": label }))
            .await
    }

    pub async fn revoke_api_key(&self, api_key_id: i64) -> Result<(), ApiError> {
        self.delete(&format!("/account/api-keys/{api_key_id}")).await
    }

    pub async fn update_settings(&self, settings: &serde_json::Value) -> Result<(), ApiError> {
        self.patch_no_response("/account/settings", settings).await
    }
}
