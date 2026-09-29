# MCP-003: Planner, Repeat & Comment Tools

## Goal

Implement 10 MCP tools for the Zealot planner, repeat/habit tracker, and comments system in `apps/mcp/src/tools/planner.rs`.

## Depends On

MCP-001, MCP-002

## Background

Zealot has three time-oriented subsystems:
- **Planner**: schedules items for specific days, weeks, months. Read-only from MCP (the schedule itself is managed in the web UI).
- **Repeat entries**: habit/recurring task tracker. Each entry has a status (Complete, Skip, Alternate, NotComplete) and optional comment. The MCP tool lets AI agents mark habits complete or add comments.
- **Comments**: timestamped notes attached to items or to a day. Useful for journaling and logging.

All dates use `YYYY-MM-DD` format. Week strings use ISO format `YYYY-WNN` (e.g. `2025-W20`).

## File: `apps/mcp/src/tools/planner.rs`

```rust
use rmcp::{
    tool,
    model::{CallToolResult, Content},
    Error as McpError,
    handler::server::tool::Parameters,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::tools::ZealotServer;
use super::api_err;

// ── Parameter structs ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DateParam {
    /// Date in YYYY-MM-DD format
    pub date: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WeekParam {
    /// ISO week string e.g. "2025-W20"
    pub week: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MonthYearParam {
    /// Month number (1–12)
    pub month: i64,
    /// Four-digit year
    pub year: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateRepeatParams {
    /// Numeric ID of the item associated with the repeat entry
    pub item_id: i64,
    /// Date in YYYY-MM-DD format
    pub date: String,
    /// Status: one of Complete, Skip, Alternate, NotComplete
    pub status: Option<String>,
    /// Optional comment text
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddCommentParams {
    /// Numeric item ID to attach the comment to
    pub item_id: i64,
    /// Timestamp in YYYY-MM-DD HH:MM:SS format
    pub timestamp: String,
    /// Comment body text
    pub content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateCommentParams {
    /// Numeric comment ID
    pub comment_id: i64,
    /// New comment body text
    pub content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CommentIdParam {
    /// Numeric comment ID
    pub comment_id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ItemIdParam {
    /// Numeric item ID
    pub item_id: i64,
}

// ── Tool implementations ─────────────────────────────────────────────────────

impl ZealotServer {
    #[tool(description = "Get all items scheduled for a specific day in the planner. Date format: YYYY-MM-DD.")]
    pub async fn get_day_plan(&self, Parameters(p): Parameters<DateParam>) -> Result<CallToolResult, McpError> {
        let items: serde_json::Value = self.client.get(&format!("/planner/day/{}", p.date)).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&items).unwrap_or_default())]))
    }

    #[tool(description = "Get all items scheduled for a specific week. Week format: YYYY-WNN (e.g. '2025-W20').")]
    pub async fn get_week_plan(&self, Parameters(p): Parameters<WeekParam>) -> Result<CallToolResult, McpError> {
        let items: serde_json::Value = self.client.get(&format!("/planner/week/{}", p.week)).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&items).unwrap_or_default())]))
    }

    #[tool(description = "Get all items scheduled for a specific month and year. Month is 1–12.")]
    pub async fn get_month_plan(&self, Parameters(p): Parameters<MonthYearParam>) -> Result<CallToolResult, McpError> {
        let items: serde_json::Value = self.client.get(&format!("/planner/month/{}/year/{}", p.month, p.year)).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&items).unwrap_or_default())]))
    }

    #[tool(description = "Get repeat/habit entries for a specific day. Shows completion status for each habit. Date format: YYYY-MM-DD.")]
    pub async fn get_repeat_entries(&self, Parameters(p): Parameters<DateParam>) -> Result<CallToolResult, McpError> {
        let entries: serde_json::Value = self.client.get(&format!("/repeat/day/{}", p.date)).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&entries).unwrap_or_default())]))
    }

    #[tool(description = "Update the status of a repeat/habit entry for a given item and date. Status must be one of: Complete, Skip, Alternate, NotComplete.")]
    pub async fn update_repeat_status(&self, Parameters(p): Parameters<UpdateRepeatParams>) -> Result<CallToolResult, McpError> {
        let body = json!({
            "item_id": p.item_id,
            "date": p.date,
            "status": p.status,
            "comment": p.comment,
        });
        self.client.put::<_, serde_json::Value>("/repeat/status", &body).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text("repeat status updated".to_string())]))
    }

    #[tool(description = "Get all comments attached to a specific item by item ID.")]
    pub async fn get_comments_for_item(&self, Parameters(p): Parameters<ItemIdParam>) -> Result<CallToolResult, McpError> {
        let comments: serde_json::Value = self.client.get(&format!("/comment/item/{}", p.item_id)).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&comments).unwrap_or_default())]))
    }

    #[tool(description = "Get all comments logged for a specific day (journal entries). Date format: YYYY-MM-DD.")]
    pub async fn get_comments_for_day(&self, Parameters(p): Parameters<DateParam>) -> Result<CallToolResult, McpError> {
        let comments: serde_json::Value = self.client.get(&format!("/comment/day/{}", p.date)).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&comments).unwrap_or_default())]))
    }

    #[tool(description = "Add a comment to an item. Timestamp format: YYYY-MM-DD HH:MM:SS. Content is plain text or markdown.")]
    pub async fn add_comment(&self, Parameters(p): Parameters<AddCommentParams>) -> Result<CallToolResult, McpError> {
        let body = json!({
            "item_id": p.item_id,
            "timestamp": p.timestamp,
            "content": p.content,
        });
        let comment: serde_json::Value = self.client.post("/comment/", &body).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&comment).unwrap_or_default())]))
    }

    #[tool(description = "Update the body text of an existing comment by its comment ID.")]
    pub async fn update_comment(&self, Parameters(p): Parameters<UpdateCommentParams>) -> Result<CallToolResult, McpError> {
        let body = json!({ "content": p.content });
        let comment: serde_json::Value = self.client.patch(&format!("/comment/{}", p.comment_id), &body).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&comment).unwrap_or_default())]))
    }

    #[tool(description = "Delete a comment by its comment ID. This is permanent.")]
    pub async fn delete_comment(&self, Parameters(p): Parameters<CommentIdParam>) -> Result<CallToolResult, McpError> {
        self.client.delete(&format!("/comment/{}", p.comment_id)).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text("comment deleted".to_string())]))
    }
}
```

## Verify

```bash
cargo check -p zealot-mcp

# Smoke test with a running Zealot server:
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_day_plan","arguments":{"date":"2025-05-13"}}}' | \
  ZEALOT_API_KEY=mykey cargo run -p zealot-mcp 2>/dev/null | jq '.result'
```
