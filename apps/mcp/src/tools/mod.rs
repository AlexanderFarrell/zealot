pub mod analysis;
pub mod automation;
pub mod media;
pub mod planner;
pub mod time_block;
pub mod wiki;

use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::router::tool::ToolRouter,
    model::{
        GetPromptRequestParams, GetPromptResult, Implementation, ListPromptsResult,
        PaginatedRequestParams, ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
};

use crate::{
    client::{ApiError, ZealotClient},
    config::Config,
};

const SERVER_INSTRUCTIONS: &str = "\
Zealot personal wiki and planner MCP v2. Use tools to read and write wiki items, \
manage planner dates, habits, comments, time blocks, media, schema, automation rules, \
and wiki analysis. Items have numeric ids, exact titles, ZealotScript/markdown content, \
JSON attributes, types, and graph links. Item parameters named `item` accept an id \
(`42` or `#42`) or an exact title.

Prefer browse-then-read. Multi-item tools return compact summary JSON by default: \
`browse_items`, `search_items`, `filter_items`, `get_linked_items`, and `get_plan` \
support `detail:\"meta\"`, `detail:\"summary\"`, or `detail:\"full\"`. Use \
`get_item` for one full item, or `get_item_outline` before reading long content. \
Paged list envelopes are `{count,next_offset,items}`; pass `offset:next_offset` \
until it is absent.

Dates are `YYYY-MM-DD`, ISO weeks are `YYYY-Wnn`, months are `YYYY-MM`, years are \
`YYYY`, and time-block times are `HH:MM`. Start daily planning with `day_dashboard`. \
Habit statuses are `Complete`, `Skip`, `Alternate`, and `Not Complete`. Use \
`add_journal_entry` only when the server has `ZEALOT_JOURNAL_ITEM` configured. \
Check `list_item_types` and `list_attribute_kinds` before inventing schema. \
Read before overwriting item content; use `append_to_item` for log-style additions.";

#[derive(Clone)]
pub struct ZealotServer {
    pub client: ZealotClient,
    pub journal_item: Option<String>,
}

impl ZealotServer {
    pub fn new(config: &Config) -> Self {
        Self {
            client: ZealotClient::new(&config.url, &config.api_key),
            journal_item: config.journal_item.clone(),
        }
    }

    pub fn tool_router() -> ToolRouter<Self> {
        let mut router = Self::wiki_tool_router();
        router.merge(Self::planner_tool_router());
        router.merge(Self::automation_tool_router());
        router.merge(Self::media_tool_router());
        router.merge(Self::time_block_tool_router());
        router.merge(Self::analysis_tool_router());
        router
    }
}

/// Map an upstream API error to an MCP error, naming the resource that was
/// being accessed so agents get an actionable message. 4xx errors are
/// `invalid_params` (the agent can correct its call); 5xx are `internal_error`.
pub fn err_ctx(resource: &str, e: ApiError) -> McpError {
    match e {
        ApiError::NotFound => McpError::invalid_params(format!("{resource} not found"), None),
        ApiError::Http { status, message } if status.is_client_error() => {
            McpError::invalid_params(format!("{resource}: {status}: {message}"), None)
        }
        ApiError::Http { status, message } => {
            McpError::internal_error(format!("{resource}: upstream {status}: {message}"), None)
        }
        e => McpError::internal_error(format!("{resource}: {e}"), None),
    }
}

/// Parse a `YYYY-MM-DD` date string into a friendly `invalid_params` error on failure.
pub fn parse_date(s: &str) -> Result<chrono::NaiveDate, McpError> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| {
        McpError::invalid_params(format!("invalid date '{s}' (expected YYYY-MM-DD)"), None)
    })
}

#[rmcp::tool_handler]
#[rmcp::prompt_handler]
impl ServerHandler for ZealotServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .build(),
        )
        .with_server_info(Implementation::new("zealot-mcp", env!("CARGO_PKG_VERSION")))
        .with_instructions(SERVER_INSTRUCTIONS)
    }
}
