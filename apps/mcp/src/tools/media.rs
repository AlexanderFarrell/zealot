use rmcp::{
    model::{CallToolResult, Content},
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::tools::{ZealotServer, api_err};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MediaPathParam {
    /// Relative path to a file or directory (empty string or omit for root)
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateFolderParams {
    /// Full relative path of the folder to create (e.g. "images/2025")
    pub folder: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteMediaParams {
    /// Relative path to the file or directory to delete
    pub path: String,
}

fn pretty(v: serde_json::Value) -> String {
    serde_json::to_string_pretty(&v).unwrap_or_default()
}

#[rmcp::tool_router(router = media_tool_router, vis = "pub")]
impl ZealotServer {
    #[rmcp::tool(description = "List the contents of the media file system. Pass an empty path for the root, or a relative path for a subdirectory. Returns file/folder metadata.")]
    pub async fn list_media(
        &self,
        Parameters(p): Parameters<MediaPathParam>,
    ) -> Result<CallToolResult, McpError> {
        let path = p.path.unwrap_or_default();
        let api_path = if path.is_empty() {
            "/media/".to_string()
        } else {
            format!("/media/{}", path.trim_start_matches('/'))
        };
        let listing: serde_json::Value = self.client.get(&api_path).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(listing))]))
    }

    #[rmcp::tool(description = "Create a new folder in the media file system. The folder path is relative (e.g. 'images/2025').")]
    pub async fn create_media_folder(
        &self,
        Parameters(p): Parameters<CreateFolderParams>,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({ "folder": p.folder });
        let _: serde_json::Value =
            self.client.post("/media/mkdir", &body).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "folder '{}' created",
            p.folder
        ))]))
    }

    #[rmcp::tool(description = "Delete a file or directory from the media file system. The path is relative. Directories must be empty before deletion.")]
    pub async fn delete_media(
        &self,
        Parameters(p): Parameters<DeleteMediaParams>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .delete(&format!("/media/{}", p.path.trim_start_matches('/')))
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "'{}' deleted",
            p.path
        ))]))
    }
}
