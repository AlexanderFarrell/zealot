use rmcp::{ErrorData as McpError, handler::server::wrapper::Parameters, model::CallToolResult};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use zealot_domain::time_block::{CreateTimeBlockDto, UpdateTimeBlockDto};

use crate::{
    output,
    tools::{ZealotServer, err_ctx, parse_date},
};

/// Format minutes-from-midnight as `H:MM`.
pub fn format_clock(min: i32) -> String {
    format!("{}:{:02}", min / 60, min % 60)
}

/// Parse an `H:MM` or `HH:MM` clock string into minutes-from-midnight.
pub fn parse_clock(s: &str) -> Result<i32, McpError> {
    let bad = || McpError::invalid_params(format!("invalid time '{s}' (expected HH:MM)"), None);
    let (h, m) = s.split_once(':').ok_or_else(bad)?;
    let h: i32 = h.trim().parse().map_err(|_| bad())?;
    let m: i32 = m.trim().parse().map_err(|_| bad())?;
    if !(0..=23).contains(&h) || !(0..=59).contains(&m) {
        return Err(bad());
    }
    Ok(h * 60 + m)
}

fn block_row(b: &zealot_domain::time_block::TimeBlockDto) -> serde_json::Value {
    json!({
        "block_id": b.block_id,
        "item_id": b.item.item_id,
        "title": b.item.title,
        "date": b.date,
        "start": format_clock(b.start_min),
        "end": format_clock(b.end_min),
        "note": b.note,
    })
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetTimeBlocksParams {
    /// Start date (inclusive), YYYY-MM-DD. Ignored if `item` is set.
    pub start_date: Option<String>,
    /// End date (inclusive), YYYY-MM-DD (default: same as start_date). Ignored if `item` is set.
    pub end_date: Option<String>,
    /// Item ID or exact title — if set, returns all blocks for this item across all dates
    pub item: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateTimeBlockParams {
    /// Item ID or exact title to attach the block to
    pub item: String,
    /// Date in YYYY-MM-DD format
    pub date: String,
    /// Start time as HH:MM (e.g. "9:00")
    pub start: String,
    /// End time as HH:MM (e.g. "10:30")
    pub end: String,
    /// Optional note text for this block
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateTimeBlockParams {
    /// Numeric block ID to update
    pub block_id: i64,
    /// New date in YYYY-MM-DD format (optional)
    pub date: Option<String>,
    /// New start time as HH:MM (optional)
    pub start: Option<String>,
    /// New end time as HH:MM (optional)
    pub end: Option<String>,
    /// New note text (optional)
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteTimeBlockParams {
    pub block_id: i64,
}

#[rmcp::tool_router(router = time_block_tool_router, vis = "pub")]
impl ZealotServer {
    #[rmcp::tool(
        description = "Get time blocks. Pass `item` to get all blocks for one item across all dates, or `start_date`/`end_date` (inclusive range, end defaults to start) to get blocks in a date range. Times are HH:MM."
    )]
    pub async fn get_time_blocks(
        &self,
        Parameters(p): Parameters<GetTimeBlocksParams>,
    ) -> Result<CallToolResult, McpError> {
        let blocks = if let Some(item) = &p.item {
            let existing = self.resolve_item(item).await?;
            self.client
                .time_blocks_for_item(existing.item_id)
                .await
                .map_err(|e| err_ctx(&format!("time blocks for item #{}", existing.item_id), e))?
        } else {
            let Some(start_str) = &p.start_date else {
                return Err(McpError::invalid_params(
                    "pass either `item` or `start_date`",
                    None,
                ));
            };
            let start = parse_date(start_str)?;
            let end = match &p.end_date {
                Some(e) => parse_date(e)?,
                None => start,
            };
            self.client
                .time_blocks_for_range(start, end)
                .await
                .map_err(|e| err_ctx("time blocks", e))?
        };
        let rows: Vec<_> = blocks.iter().map(block_row).collect();
        Ok(output::json_result(&rows))
    }

    #[rmcp::tool(
        description = "Create a new time block for an item. Times are HH:MM (e.g. start:\"9:00\", end:\"10:30\")."
    )]
    pub async fn create_time_block(
        &self,
        Parameters(p): Parameters<CreateTimeBlockParams>,
    ) -> Result<CallToolResult, McpError> {
        let existing = self.resolve_item(&p.item).await?;
        let start_min = parse_clock(&p.start)?;
        let end_min = parse_clock(&p.end)?;
        let dto = CreateTimeBlockDto {
            item_id: existing.item_id,
            date: p.date,
            start_min,
            end_min,
            note: p.note,
        };
        let block = self
            .client
            .create_time_block(&dto)
            .await
            .map_err(|e| err_ctx(&format!("time block for item #{}", existing.item_id), e))?;
        Ok(output::json_result(&block_row(&block)))
    }

    #[rmcp::tool(
        description = "Update an existing time block. All fields are optional — only provided fields are changed. Times are HH:MM."
    )]
    pub async fn update_time_block(
        &self,
        Parameters(p): Parameters<UpdateTimeBlockParams>,
    ) -> Result<CallToolResult, McpError> {
        let start_min = p.start.as_deref().map(parse_clock).transpose()?;
        let end_min = p.end.as_deref().map(parse_clock).transpose()?;
        let dto = UpdateTimeBlockDto {
            block_id: p.block_id,
            date: p.date,
            start_min,
            end_min,
            note: p.note,
        };
        self.client
            .update_time_block(&dto)
            .await
            .map_err(|e| err_ctx(&format!("time block #{}", p.block_id), e))?;
        Ok(output::json_result(
            &json!({"block_id": p.block_id, "status": "updated"}),
        ))
    }

    #[rmcp::tool(description = "Delete a time block permanently by its block ID.")]
    pub async fn delete_time_block(
        &self,
        Parameters(p): Parameters<DeleteTimeBlockParams>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .delete_time_block(p.block_id)
            .await
            .map_err(|e| err_ctx(&format!("time block #{}", p.block_id), e))?;
        Ok(output::json_result(&json!({"deleted": p.block_id})))
    }
}
