# MCP-005: Media Tools

## Goal

Implement 3 MCP tools for Zealot's media file system in `apps/mcp/src/tools/media.rs`. File upload is excluded — binary content is not meaningful in MCP text responses.

## Depends On

MCP-001, MCP-002

## Background

Zealot manages a virtual file system for attachments and media. The API returns directory listings as `FileStatDto` arrays with fields: `name`, `size`, `modified`, `is_dir`. Paths are relative (e.g. `"images/photo.jpg"` or `""` for root).

## File: `apps/mcp/src/tools/media.rs`

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

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MediaPathParam {
    /// Relative path to a file or directory (empty string for root)
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateFolderParams {
    /// Full path of the folder to create (e.g. "images/2025")
    pub folder: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteMediaParams {
    /// Relative path to the file or directory to delete
    pub path: String,
}

impl ZealotServer {
    #[tool(description = "List the contents of the media file system. Pass an empty path for the root, or a relative path for a subdirectory. Returns file/folder metadata including name, size, and modification date.")]
    pub async fn list_media(&self, Parameters(p): Parameters<MediaPathParam>) -> Result<CallToolResult, McpError> {
        let path = p.path.unwrap_or_default();
        let api_path = if path.is_empty() {
            "/media/".to_string()
        } else {
            format!("/media/{}", path.trim_start_matches('/'))
        };
        let listing: serde_json::Value = self.client.get(&api_path).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&listing).unwrap_or_default())]))
    }

    #[tool(description = "Create a new folder in the media file system. The folder path is relative (e.g. 'images/2025').")]
    pub async fn create_media_folder(&self, Parameters(p): Parameters<CreateFolderParams>) -> Result<CallToolResult, McpError> {
        let body = json!({ "folder": p.folder });
        let _: serde_json::Value = self.client.post("/media/mkdir", &body).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(format!("folder '{}' created", p.folder))]))
    }

    #[tool(description = "Delete a file or directory from the media file system. The path is relative. Directories must be empty before deletion.")]
    pub async fn delete_media(&self, Parameters(p): Parameters<DeleteMediaParams>) -> Result<CallToolResult, McpError> {
        self.client.delete(&format!("/media/{}", p.path.trim_start_matches('/'))).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(format!("'{}' deleted", p.path))]))
    }
}
```

## Verify

```bash
cargo check -p zealot-mcp

# List root media:
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list_media","arguments":{"path":""}}}' | \
  ZEALOT_API_KEY=mykey cargo run -p zealot-mcp 2>/dev/null | jq '.result'
```
