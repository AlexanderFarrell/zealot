use rmcp::{ErrorData as McpError, handler::server::wrapper::Parameters, model::CallToolResult};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use zealot_domain::{comment::AddCommentDto, repeat::UpdateRepeatEntryDto};

use crate::{
    output::{self, DEFAULT_LIMIT, Detail},
    tools::{ZealotServer, err_ctx, parse_date},
};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DayDashboardParams {
    /// Date in YYYY-MM-DD format (default: today)
    pub date: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetPlanParams {
    /// A day (YYYY-MM-DD), ISO week (YYYY-Wnn), month (YYYY-MM), or year (YYYY)
    pub period: String,
    /// Output detail: "meta", "summary" (default), or "full"
    #[serde(default)]
    pub detail: Detail,
    /// Max results (default 50) — only applied client-side for month/year periods
    pub limit: Option<i64>,
    /// Offset for pagination — only applied client-side for month/year periods
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListHabitsParams {
    /// Output detail: "meta", "summary" (default), or "full"
    #[serde(default)]
    pub detail: Detail,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct HabitEntriesParams {
    /// Start date (inclusive), YYYY-MM-DD
    pub start_date: String,
    /// End date (inclusive), YYYY-MM-DD (default: same as start_date)
    pub end_date: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetHabitStatusParams {
    /// Item ID or exact title
    pub item: String,
    /// Date in YYYY-MM-DD format
    pub date: String,
    /// Status: Complete, Skip, Alternate, or "Not Complete"
    pub status: Option<String>,
    /// Optional comment text
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetCommentsParams {
    /// Item ID or exact title — mutually exclusive with `date`
    pub item: Option<String>,
    /// Date in YYYY-MM-DD format — mutually exclusive with `item`
    pub date: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddCommentParams {
    /// Item ID or exact title
    pub item: String,
    /// Comment body text
    pub content: String,
    /// Timestamp in YYYY-MM-DD HH:MM:SS format (default: now)
    pub timestamp: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddJournalEntryParams {
    /// Journal entry text
    pub content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateCommentParams {
    pub comment_id: i64,
    pub content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CommentIdParam {
    pub comment_id: i64,
}

fn today_str() -> String {
    chrono::Local::now()
        .date_naive()
        .format("%Y-%m-%d")
        .to_string()
}

fn now_timestamp() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Classify a period string by shape: day, ISO week, month, or year.
enum Period {
    Day(chrono::NaiveDate),
    Week(String),
    Month(u32, i32),
    Year(i32),
}

fn parse_period(s: &str) -> Result<Period, McpError> {
    let bad = || {
        McpError::invalid_params(
            format!("invalid period '{s}' (expected YYYY-MM-DD, YYYY-Wnn, YYYY-MM, or YYYY)"),
            None,
        )
    };
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(Period::Day(d));
    }
    if let Some((y, w)) = s.split_once("-W") {
        let year: i32 = y.parse().map_err(|_| bad())?;
        let _week: u32 = w.parse().map_err(|_| bad())?;
        let _ = year;
        return Ok(Period::Week(s.to_string()));
    }
    if let Some((y, m)) = s.split_once('-') {
        let year: i32 = y.parse().map_err(|_| bad())?;
        let month: u32 = m.parse().map_err(|_| bad())?;
        if !(1..=12).contains(&month) {
            return Err(bad());
        }
        return Ok(Period::Month(month, year));
    }
    if s.len() == 4 {
        let year: i32 = s.parse().map_err(|_| bad())?;
        return Ok(Period::Year(year));
    }
    Err(bad())
}

#[rmcp::tool_router(router = planner_tool_router, vis = "pub")]
impl ZealotServer {
    #[rmcp::tool(
        description = "One-call snapshot of a day: planner items, habit statuses, time blocks, and journal comments. Best starting point for 'what's on my plate today'. Date format YYYY-MM-DD, defaults to today."
    )]
    pub async fn day_dashboard(
        &self,
        Parameters(p): Parameters<DayDashboardParams>,
    ) -> Result<CallToolResult, McpError> {
        let date_str = p.date.unwrap_or_else(today_str);
        let date = parse_date(&date_str)?;

        let (plan, habits, blocks, comments) = tokio::join!(
            self.client.planner_day(date),
            self.client.repeats_for_day(date),
            self.client.time_blocks_for_day(date),
            self.client.comments_for_day(date),
        );
        let plan = plan.map_err(|e| err_ctx("day plan", e))?;
        let habits = habits.map_err(|e| err_ctx("habit entries", e))?;
        let blocks = blocks.map_err(|e| err_ctx("time blocks", e))?;
        let comments = comments.map_err(|e| err_ctx("comments", e))?;

        let plan_summaries: Vec<_> = plan
            .iter()
            .map(|i| output::ItemSummary::project(i, Detail::Summary))
            .collect();
        let habit_rows: Vec<_> = habits
            .iter()
            .map(|h| {
                json!({
                    "item_id": h.item.item_id,
                    "title": h.item.title,
                    "status": h.status,
                    "comment": h.comment,
                })
            })
            .collect();
        let block_rows: Vec<_> = blocks
            .iter()
            .map(|b| {
                json!({
                    "block_id": b.block_id,
                    "item_id": b.item.item_id,
                    "title": b.item.title,
                    "start": crate::tools::time_block::format_clock(b.start_min),
                    "end": crate::tools::time_block::format_clock(b.end_min),
                    "note": b.note,
                })
            })
            .collect();
        let journal_rows: Vec<_> = comments
            .iter()
            .map(|c| {
                json!({
                    "comment_id": c.comment_id,
                    "item_id": c.item.item_id,
                    "item_title": c.item.title,
                    "timestamp": c.timestamp,
                    "content": c.content,
                })
            })
            .collect();

        Ok(output::json_result(&json!({
            "date": date_str,
            "plan": plan_summaries,
            "habits": habit_rows,
            "time_blocks": block_rows,
            "journal": journal_rows,
        })))
    }

    #[rmcp::tool(
        description = "Get planned items for a period. `period` shape determines granularity: YYYY-MM-DD for a day, YYYY-Wnn for an ISO week (e.g. \"2026-W27\"), YYYY-MM for a month, or YYYY for a year. Returns compact summaries by default; pass detail:\"full\" for complete content."
    )]
    pub async fn get_plan(
        &self,
        Parameters(p): Parameters<GetPlanParams>,
    ) -> Result<CallToolResult, McpError> {
        let limit = p.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, 500);
        let offset = p.offset.unwrap_or(0).max(0);
        let period = parse_period(&p.period)?;
        let items = match period {
            Period::Day(d) => self
                .client
                .planner_day(d)
                .await
                .map_err(|e| err_ctx("day plan", e))?,
            Period::Week(w) => self
                .client
                .planner_week(&w)
                .await
                .map_err(|e| err_ctx("week plan", e))?,
            Period::Month(m, y) => self
                .client
                .planner_month(m, y)
                .await
                .map_err(|e| err_ctx("month plan", e))?,
            Period::Year(y) => self
                .client
                .planner_year(y)
                .await
                .map_err(|e| err_ctx("year plan", e))?,
        };
        // Planner endpoints are unbounded server-side; slice client-side.
        let start = offset.max(0) as usize;
        let end = start.saturating_add(limit.max(0) as usize);
        let sliced = items.into_iter().skip(start).take(end - start).collect();
        Ok(output::item_page(sliced, p.detail, offset, limit))
    }

    #[rmcp::tool(description = "List all items enrolled in the habit/repeat tracker.")]
    pub async fn list_habits(
        &self,
        Parameters(p): Parameters<ListHabitsParams>,
    ) -> Result<CallToolResult, McpError> {
        let items = self
            .client
            .repeat_items()
            .await
            .map_err(|e| err_ctx("habits", e))?;
        let summaries: Vec<_> = items
            .iter()
            .map(|i| output::ItemSummary::project(i, p.detail))
            .collect();
        Ok(output::json_result(&summaries))
    }

    #[rmcp::tool(
        description = "Get habit/repeat completion entries over a date range (inclusive). Rows are compact: {item_id, title, date, status, comment}. Omit end_date to query a single day."
    )]
    pub async fn get_habit_entries(
        &self,
        Parameters(p): Parameters<HabitEntriesParams>,
    ) -> Result<CallToolResult, McpError> {
        let start = parse_date(&p.start_date)?;
        let end = match &p.end_date {
            Some(e) => parse_date(e)?,
            None => start,
        };
        let entries = self
            .client
            .repeats_for_range(start, end)
            .await
            .map_err(|e| err_ctx("habit entries", e))?;
        let rows: Vec<_> = entries
            .iter()
            .map(|e| {
                json!({
                    "item_id": e.item.item_id,
                    "title": e.item.title,
                    "date": e.date,
                    "status": e.status,
                    "comment": e.comment,
                })
            })
            .collect();
        Ok(output::json_result(&rows))
    }

    #[rmcp::tool(
        description = "Update the status of a habit/repeat entry for a given item and date. Status must be one of: Complete, Skip, Alternate, \"Not Complete\"."
    )]
    pub async fn set_habit_status(
        &self,
        Parameters(p): Parameters<SetHabitStatusParams>,
    ) -> Result<CallToolResult, McpError> {
        let existing = self.resolve_item(&p.item).await?;
        let dto = UpdateRepeatEntryDto {
            item_id: existing.item_id,
            date: p.date.clone(),
            status: p.status,
            comment: p.comment,
        };
        self.client
            .set_repeat_status(&dto)
            .await
            .map_err(|e| err_ctx(&format!("habit entry for item #{}", existing.item_id), e))?;
        Ok(output::json_result(
            &json!({"item_id": existing.item_id, "date": p.date, "status": "updated"}),
        ))
    }

    #[rmcp::tool(
        description = "Get comments (journal entries) either for one item or for one day. Pass exactly one of `item` or `date`."
    )]
    pub async fn get_comments(
        &self,
        Parameters(p): Parameters<GetCommentsParams>,
    ) -> Result<CallToolResult, McpError> {
        let comments = match (p.item, p.date) {
            (Some(item), None) => {
                let existing = self.resolve_item(&item).await?;
                self.client
                    .comments_for_item(existing.item_id)
                    .await
                    .map_err(|e| err_ctx(&format!("comments for item #{}", existing.item_id), e))?
            }
            (None, Some(date)) => {
                let d = parse_date(&date)?;
                self.client
                    .comments_for_day(d)
                    .await
                    .map_err(|e| err_ctx("comments", e))?
            }
            _ => {
                return Err(McpError::invalid_params(
                    "pass exactly one of `item` or `date`",
                    None,
                ));
            }
        };
        let rows: Vec<_> = comments
            .iter()
            .map(|c| {
                json!({
                    "comment_id": c.comment_id,
                    "item_id": c.item.item_id,
                    "item_title": c.item.title,
                    "timestamp": c.timestamp,
                    "content": c.content,
                })
            })
            .collect();
        Ok(output::json_result(&rows))
    }

    #[rmcp::tool(
        description = "Add a comment to an item. Timestamp format: YYYY-MM-DD HH:MM:SS (defaults to now)."
    )]
    pub async fn add_comment(
        &self,
        Parameters(p): Parameters<AddCommentParams>,
    ) -> Result<CallToolResult, McpError> {
        let existing = self.resolve_item(&p.item).await?;
        let dto = AddCommentDto {
            item_id: existing.item_id,
            timestamp: p.timestamp.unwrap_or_else(now_timestamp),
            content: p.content,
        };
        let comment = self
            .client
            .add_comment(&dto)
            .await
            .map_err(|e| err_ctx(&format!("comment on item #{}", existing.item_id), e))?;
        Ok(output::json_result(&comment))
    }

    #[rmcp::tool(
        description = "Quick-capture a journal entry: adds a now-stamped comment to your configured journal item. Requires ZEALOT_JOURNAL_ITEM (or --journal-item) to be set on the server."
    )]
    pub async fn add_journal_entry(
        &self,
        Parameters(p): Parameters<AddJournalEntryParams>,
    ) -> Result<CallToolResult, McpError> {
        let Some(journal_ref) = self.journal_item.clone() else {
            return Err(McpError::invalid_params(
                "no journal item configured — set ZEALOT_JOURNAL_ITEM (or --journal-item) on the zealot-mcp server",
                None,
            ));
        };
        let existing = self.resolve_item(&journal_ref).await?;
        let dto = AddCommentDto {
            item_id: existing.item_id,
            timestamp: now_timestamp(),
            content: p.content,
        };
        let comment = self
            .client
            .add_comment(&dto)
            .await
            .map_err(|e| err_ctx("journal entry", e))?;
        Ok(output::json_result(&comment))
    }

    #[rmcp::tool(description = "Update the body text of an existing comment.")]
    pub async fn update_comment(
        &self,
        Parameters(p): Parameters<UpdateCommentParams>,
    ) -> Result<CallToolResult, McpError> {
        let dto = zealot_domain::comment::UpdateCommentDto {
            comment_id: p.comment_id,
            item_id: None,
            timestamp: None,
            content: Some(p.content),
        };
        let comment = self
            .client
            .update_comment(&dto)
            .await
            .map_err(|e| err_ctx(&format!("comment #{}", p.comment_id), e))?;
        Ok(output::json_result(&comment))
    }

    #[rmcp::tool(description = "Delete a comment. This is permanent.")]
    pub async fn delete_comment(
        &self,
        Parameters(p): Parameters<CommentIdParam>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .delete_comment(p.comment_id)
            .await
            .map_err(|e| err_ctx(&format!("comment #{}", p.comment_id), e))?;
        Ok(output::json_result(&json!({"deleted": p.comment_id})))
    }
}
