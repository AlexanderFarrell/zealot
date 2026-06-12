use rmcp::{
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content},
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::tools::{ZealotServer, api_err};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TimeBlockDateParam {
    /// Date in YYYY-MM-DD format
    pub date: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TimeBlockRangeParam {
    /// Start date (inclusive) in YYYY-MM-DD format
    pub start_date: String,
    /// End date (inclusive) in YYYY-MM-DD format
    pub end_date: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TimeBlockItemParam {
    /// Numeric item ID
    pub item_id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateTimeBlockParam {
    /// Numeric item ID to attach the block to
    pub item_id: i64,
    /// Date in YYYY-MM-DD format
    pub date: String,
    /// Start time as minutes since midnight (0–1439)
    pub start_min: i32,
    /// End time as minutes since midnight (0–1439)
    pub end_min: i32,
    /// Optional note text for this block
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateTimeBlockParam {
    /// Numeric block ID to update
    pub block_id: i64,
    /// New date in YYYY-MM-DD format (optional)
    pub date: Option<String>,
    /// New start time as minutes since midnight (optional)
    pub start_min: Option<i32>,
    /// New end time as minutes since midnight (optional)
    pub end_min: Option<i32>,
    /// New note text (optional)
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteTimeBlockParam {
    /// Numeric block ID to delete
    pub block_id: i64,
}

fn pretty(v: serde_json::Value) -> String {
    serde_json::to_string_pretty(&v).unwrap_or_default()
}

#[rmcp::tool_router(router = time_block_tool_router, vis = "pub")]
impl ZealotServer {
    #[rmcp::tool(
        description = "Get all time blocks scheduled for a specific day. Returns blocks with item details and start/end times in minutes since midnight. Date format: YYYY-MM-DD."
    )]
    pub async fn get_time_blocks_for_day(
        &self,
        Parameters(p): Parameters<TimeBlockDateParam>,
    ) -> Result<CallToolResult, McpError> {
        let blocks: serde_json::Value = self
            .client
            .get(&format!("/time_block/day/{}", p.date))
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(blocks))]))
    }

    #[rmcp::tool(
        description = "Get all time blocks across a date range (inclusive). Returns blocks sorted by date and start time. Date format: YYYY-MM-DD."
    )]
    pub async fn get_time_blocks_for_range(
        &self,
        Parameters(p): Parameters<TimeBlockRangeParam>,
    ) -> Result<CallToolResult, McpError> {
        let blocks: serde_json::Value = self
            .client
            .get(&format!(
                "/time_block/range?start={}&end={}",
                p.start_date, p.end_date
            ))
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(blocks))]))
    }

    #[rmcp::tool(
        description = "Get all time blocks for a specific item across all dates. Useful to see the full schedule for one item."
    )]
    pub async fn get_time_blocks_for_item(
        &self,
        Parameters(p): Parameters<TimeBlockItemParam>,
    ) -> Result<CallToolResult, McpError> {
        let blocks: serde_json::Value = self
            .client
            .get(&format!("/time_block/item/{}", p.item_id))
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(blocks))]))
    }

    #[rmcp::tool(
        description = "Create a new time block for an item. Times are in minutes since midnight (e.g. 9:00 = 540, 17:30 = 1050). Returns the created block."
    )]
    pub async fn create_time_block(
        &self,
        Parameters(p): Parameters<CreateTimeBlockParam>,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({
            "item_id":   p.item_id,
            "date":      p.date,
            "start_min": p.start_min,
            "end_min":   p.end_min,
            "note":      p.note,
        });
        let block: serde_json::Value =
            self.client.post("/time_block", &body).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(block))]))
    }

    #[rmcp::tool(
        description = "Update an existing time block. All fields are optional — only provided fields are changed."
    )]
    pub async fn update_time_block(
        &self,
        Parameters(p): Parameters<UpdateTimeBlockParam>,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({
            "block_id":  p.block_id,
            "date":      p.date,
            "start_min": p.start_min,
            "end_min":   p.end_min,
            "note":      p.note,
        });
        self.client
            .patch_no_response(&format!("/time_block/{}", p.block_id), &body)
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(
            "time block updated".to_string(),
        )]))
    }

    #[rmcp::tool(description = "Delete a time block permanently by its block ID.")]
    pub async fn delete_time_block(
        &self,
        Parameters(p): Parameters<DeleteTimeBlockParam>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .delete(&format!("/time_block/{}", p.block_id))
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(
            "time block deleted".to_string(),
        )]))
    }
}
