//! Saved View tools: run, inspect, and validate the item query held in a `Saved View` item.

use rmcp::{ErrorData as McpError, handler::server::wrapper::Parameters, model::CallToolResult};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    output::{self, Detail, Shape},
    tools::{ZealotServer, err_ctx},
};

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunViewParams {
    /// Saved View item ID or exact title
    pub view: String,
    /// Max results (default: the view's own limit, else 50; max 100)
    pub limit: Option<i64>,
    /// Offset for pagination (default 0)
    pub offset: Option<i64>,
    /// Output detail: "meta", "summary", or "full". Without `detail` or `fields`, the view's
    /// `columns` pick the attributes shown, else "summary"
    pub detail: Option<Detail>,
    /// Return only these attributes per item (plus `item_id`, `title`, `types`)
    pub fields: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ViewRefParams {
    /// Saved View item ID or exact title
    pub view: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ValidateViewParams {
    /// The view definition: the JSON object that goes inside the ```zealot-view block,
    /// `{"query": {...filter_items query...}, "group_by"?: "Parent", "columns"?: ["Status"]}`
    pub definition: serde_json::Value,
}

#[rmcp::tool_router(router = view_tool_router, vis = "pub")]
impl ZealotServer {
    #[rmcp::tool(
        description = "Run a Saved View: an item of type `Saved View` whose content holds one ```zealot-view JSON block `{\"query\": {...}, \"group_by\"?: key, \"columns\"?: [keys]}`. `query` takes everything `filter_items` takes (types, under, filters, any_of, linked_from, sort), including relative dates such as `$today`, resolved when the view runs. Returns `{view_id, title, total, count, next_offset?, items, group_by?, groups?: [{key, label, item_ids}], warnings?}`; groups cover the returned page. Items are shaped by `detail`/`fields`, defaulting to the view's `columns`."
    )]
    pub async fn run_view(
        &self,
        Parameters(p): Parameters<RunViewParams>,
    ) -> Result<CallToolResult, McpError> {
        let view_id = self.resolve_item_id(&p.view).await?;
        let result = self
            .client
            .run_view(view_id, p.limit, p.offset)
            .await
            .map_err(|e| err_ctx(&format!("Saved View #{view_id}"), e))?;
        let shape = match (p.fields, p.detail) {
            (Some(fields), _) => Shape::Fields(fields),
            (None, Some(detail)) => Shape::Detail(detail),
            (None, None) if !result.columns.is_empty() => Shape::Fields(result.columns.clone()),
            (None, None) => Shape::Detail(Detail::default()),
        };
        Ok(output::view_result(result, shape))
    }

    #[rmcp::tool(
        description = "Explain how a Saved View evaluates now: its definition, the query after relative dates are resolved (`resolved_query`), `today` and `time_zone`, the match `total`, `elapsed_ms`, and warnings (such as ignored query keys)."
    )]
    pub async fn inspect_view(
        &self,
        Parameters(p): Parameters<ViewRefParams>,
    ) -> Result<CallToolResult, McpError> {
        let view_id = self.resolve_item_id(&p.view).await?;
        let report = self
            .client
            .inspect_view(view_id)
            .await
            .map_err(|e| err_ctx(&format!("Saved View #{view_id}"), e))?;
        Ok(output::json_result(&report))
    }

    #[rmcp::tool(
        description = "Check a Saved View definition before saving it: parses it, runs its query once, and returns the same report as `inspect_view`, or an error naming the problem. To save a view, create an item with type `Saved View` whose content contains the definition in a ```zealot-view fenced block."
    )]
    pub async fn validate_view(
        &self,
        Parameters(p): Parameters<ValidateViewParams>,
    ) -> Result<CallToolResult, McpError> {
        let report = self
            .client
            .validate_view(p.definition)
            .await
            .map_err(|e| err_ctx("view definition", e))?;
        Ok(output::json_result(&report))
    }
}
