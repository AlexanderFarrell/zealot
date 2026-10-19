//! Saved Views (`/view`): run, inspect, and validate.

use serde_json::Value;
use zealot_domain::saved_view::{RunViewDto, ValidateViewDto, ViewInspectDto, ViewResultDto};

use crate::{ZealotClient, error::ApiError};

impl ZealotClient {
    /// Run a Saved View; `limit`/`offset` override the definition's paging.
    pub async fn run_view(
        &self,
        view_id: i64,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<ViewResultDto, ApiError> {
        self.post(
            &format!("/view/{view_id}/run"),
            &RunViewDto { limit, offset },
        )
        .await
    }

    /// Show how a Saved View evaluates: resolved query, total, time zone, timing, warnings.
    pub async fn inspect_view(&self, view_id: i64) -> Result<ViewInspectDto, ApiError> {
        self.get(&format!("/view/{view_id}/inspect")).await
    }

    /// Check an unsaved view definition (the JSON that goes in a ```zealot-view block).
    pub async fn validate_view(&self, definition: Value) -> Result<ViewInspectDto, ApiError> {
        self.post("/view/validate", &ValidateViewDto { definition })
            .await
    }
}
