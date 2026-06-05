pub mod automation;
pub mod media;
pub mod planner;
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

#[derive(Clone)]
pub struct ZealotServer {
    pub client: ZealotClient,
}

impl ZealotServer {
    pub fn new(config: &Config) -> Self {
        Self {
            client: ZealotClient::new(&config.url, &config.api_key),
        }
    }

    pub fn tool_router() -> ToolRouter<Self> {
        let mut router = Self::wiki_tool_router();
        router.merge(Self::planner_tool_router());
        router.merge(Self::automation_tool_router());
        router.merge(Self::media_tool_router());
        router
    }
}

pub fn api_err(e: ApiError) -> McpError {
    match e {
        ApiError::NotFound => McpError::invalid_params("not found", None),
        ApiError::Http { status, message } => {
            McpError::internal_error(format!("upstream HTTP {status}: {message}"), None)
        }
        e => McpError::internal_error(e.to_string(), None),
    }
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
        .with_server_info(Implementation::new(
            "zealot-mcp",
            env!("CARGO_PKG_VERSION"),
        ))
        .with_instructions(
            "Zealot personal wiki and planner. Use the tools to read and write wiki items, \
            manage your planner and habits, run automations, and interact with your knowledge base. \
            Items have types, attributes, relationships, and content written in ZealotScript (markdown-like). \
            Dates are always YYYY-MM-DD format.",
        )
    }
}
