use rmcp::{ErrorData as McpError, handler::server::wrapper::Parameters, model::CallToolResult};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use zealot_domain::statistic::{CreateStatisticEntryDto, UpdateStatisticEntryDto};

use crate::{
    output,
    tools::{ZealotServer, err_ctx},
};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListStatisticsParams {
    /// Optional parent item ID or exact title
    pub parent: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StatisticQueryParams {
    /// Statistic item ID or exact title
    pub statistic: String,
    /// Inclusive RFC 3339 timestamp
    pub start: Option<String>,
    /// Exclusive RFC 3339 timestamp
    pub end: Option<String>,
    /// Page size, 1-100
    pub limit: Option<i64>,
    /// Page offset
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RecordStatisticEntryParams {
    /// Statistic item ID or exact title
    pub statistic: String,
    pub value: f64,
    /// RFC 3339 timestamp; defaults to now
    pub occurred_at: Option<String>,
    /// Optional related item ID or exact title
    pub related_item: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EditStatisticEntryParams {
    pub statistic_entry_id: i64,
    pub value: Option<f64>,
    pub occurred_at: Option<String>,
    /// Related item ID or exact title
    pub related_item: Option<String>,
    /// Clear the related item
    pub clear_related_item: Option<bool>,
    pub comment: Option<String>,
    /// Clear the comment
    pub clear_comment: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteStatisticEntryParams {
    pub statistic_entry_id: i64,
}

#[rmcp::tool_router(router = statistic_tool_router, vis = "pub")]
impl ZealotServer {
    #[rmcp::tool(description = "List Statistic items, optionally restricted to a parent item.")]
    pub async fn list_statistics(
        &self,
        Parameters(p): Parameters<ListStatisticsParams>,
    ) -> Result<CallToolResult, McpError> {
        let parent_id = match p.parent {
            Some(reference) => Some(self.resolve_item(&reference).await?.item_id),
            None => None,
        };
        let items = self
            .client
            .statistic_items(parent_id)
            .await
            .map_err(|e| err_ctx("statistics", e))?;
        Ok(output::json_result(&items))
    }

    #[rmcp::tool(
        description = "Get paginated raw Statistic Entries. Start is inclusive and end is exclusive; both use RFC 3339."
    )]
    pub async fn get_statistic_entries(
        &self,
        Parameters(p): Parameters<StatisticQueryParams>,
    ) -> Result<CallToolResult, McpError> {
        let item = self.resolve_item(&p.statistic).await?;
        let page = self
            .client
            .statistic_entries(
                item.item_id,
                p.start.as_deref(),
                p.end.as_deref(),
                p.limit.unwrap_or(50),
                p.offset.unwrap_or(0),
            )
            .await
            .map_err(|e| err_ctx(&format!("Statistic Entries for #{}", item.item_id), e))?;
        Ok(output::json_result(&page))
    }

    #[rmcp::tool(
        description = "Get the UTC daily aggregate series for a Statistic. Start is inclusive and end is exclusive RFC 3339."
    )]
    pub async fn get_statistic_daily_series(
        &self,
        Parameters(p): Parameters<StatisticQueryParams>,
    ) -> Result<CallToolResult, McpError> {
        let item = self.resolve_item(&p.statistic).await?;
        let points = self
            .client
            .statistic_daily(item.item_id, p.start.as_deref(), p.end.as_deref())
            .await
            .map_err(|e| err_ctx(&format!("daily Statistic series for #{}", item.item_id), e))?;
        Ok(output::json_result(&points))
    }

    #[rmcp::tool(
        description = "Get count, first, latest, minimum, maximum, average, sum, and delta for a Statistic period."
    )]
    pub async fn get_statistic_summary(
        &self,
        Parameters(p): Parameters<StatisticQueryParams>,
    ) -> Result<CallToolResult, McpError> {
        let item = self.resolve_item(&p.statistic).await?;
        let summary = self
            .client
            .statistic_summary(item.item_id, p.start.as_deref(), p.end.as_deref())
            .await
            .map_err(|e| err_ctx(&format!("Statistic summary for #{}", item.item_id), e))?;
        Ok(output::json_result(&summary))
    }

    #[rmcp::tool(
        description = "Record a numeric Statistic Entry. occurred_at defaults to the current UTC time."
    )]
    pub async fn record_statistic_entry(
        &self,
        Parameters(p): Parameters<RecordStatisticEntryParams>,
    ) -> Result<CallToolResult, McpError> {
        let item = self.resolve_item(&p.statistic).await?;
        let related_item_id = match p.related_item {
            Some(reference) => Some(self.resolve_item(&reference).await?.item_id),
            None => None,
        };
        let entry = self
            .client
            .create_statistic_entry(
                item.item_id,
                &CreateStatisticEntryDto {
                    value: p.value,
                    occurred_at: p.occurred_at,
                    related_item_id,
                    comment: p.comment,
                },
            )
            .await
            .map_err(|e| err_ctx(&format!("Statistic Entry for #{}", item.item_id), e))?;
        Ok(output::json_result(&entry))
    }

    #[rmcp::tool(
        description = "Edit supplied fields on a Statistic Entry. Use clear flags to remove contextual fields."
    )]
    pub async fn edit_statistic_entry(
        &self,
        Parameters(p): Parameters<EditStatisticEntryParams>,
    ) -> Result<CallToolResult, McpError> {
        let related_item_id = if p.clear_related_item.unwrap_or(false) {
            Some(None)
        } else if let Some(reference) = p.related_item {
            Some(Some(self.resolve_item(&reference).await?.item_id))
        } else {
            None
        };
        let comment = if p.clear_comment.unwrap_or(false) {
            Some(None)
        } else {
            p.comment.map(Some)
        };
        let entry = self
            .client
            .update_statistic_entry(
                p.statistic_entry_id,
                &UpdateStatisticEntryDto {
                    value: p.value,
                    occurred_at: p.occurred_at,
                    related_item_id,
                    comment,
                },
            )
            .await
            .map_err(|e| err_ctx(&format!("Statistic Entry #{}", p.statistic_entry_id), e))?;
        Ok(output::json_result(&entry))
    }

    #[rmcp::tool(description = "Delete a Statistic Entry permanently.")]
    pub async fn delete_statistic_entry(
        &self,
        Parameters(p): Parameters<DeleteStatisticEntryParams>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .delete_statistic_entry(p.statistic_entry_id)
            .await
            .map_err(|e| err_ctx(&format!("Statistic Entry #{}", p.statistic_entry_id), e))?;
        Ok(output::json_result(
            &json!({"deleted": p.statistic_entry_id}),
        ))
    }
}
