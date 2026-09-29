use base64::Engine;
use rmcp::{
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content},
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use zealot_client::types::MediaEntry;

use crate::{
    output,
    tools::{ZealotServer, err_ctx},
};

const MAX_TEXT_BYTES: usize = 100 * 1024;
const MAX_INLINE_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MediaPathParam {
    /// Relative path to a file or directory (empty or omit for root)
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetMediaParams {
    /// Relative path to a file or directory
    pub path: String,
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum MediaEncoding {
    Text,
    Base64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UploadMediaParams {
    /// Full relative path for the new file, e.g. "images/2025/photo.png"
    pub path: String,
    /// File content
    pub content: String,
    /// How `content` is encoded: "text" (written as-is) or "base64" (decoded first, for binary files)
    pub encoding: MediaEncoding,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateFolderParams {
    /// Full relative path of the folder to create (e.g. "images/2025")
    pub folder: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RenameMediaParams {
    /// Current relative path of the file or folder
    pub old_location: String,
    /// New name (just the filename/folder name, not a full path)
    pub new_name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteMediaParams {
    /// Relative path to the file or directory to delete
    pub path: String,
}

fn is_text_content_type(ct: Option<&str>) -> bool {
    ct.is_some_and(|ct| ct.starts_with("text/") || ct.starts_with("application/json"))
}

fn is_image_content_type(ct: Option<&str>) -> bool {
    ct.is_some_and(|ct| ct.starts_with("image/"))
}

#[rmcp::tool_router(router = media_tool_router, vis = "pub")]
impl ZealotServer {
    #[rmcp::tool(
        description = "List the contents of a media directory. Pass an empty path for the root, or a relative path for a subdirectory."
    )]
    pub async fn list_media(
        &self,
        Parameters(p): Parameters<MediaPathParam>,
    ) -> Result<CallToolResult, McpError> {
        let path = p.path.unwrap_or_default();
        let files = self
            .client
            .media_list(&path)
            .await
            .map_err(|e| err_ctx(&format!("media at '{path}'"), e))?;
        Ok(output::json_result(&files))
    }

    #[rmcp::tool(
        description = "Get a media file or directory listing. Directories return file metadata. Images return inline image content. Text/JSON files return their text (truncated past 100 KB). Other or large (>2 MB) files return metadata only — use the CLI or web UI to download them."
    )]
    pub async fn get_media(
        &self,
        Parameters(p): Parameters<GetMediaParams>,
    ) -> Result<CallToolResult, McpError> {
        let entry = self
            .client
            .media_get(&p.path)
            .await
            .map_err(|e| err_ctx(&format!("media at '{}'", p.path), e))?;
        match entry {
            MediaEntry::Directory(files) => Ok(output::json_result(&files)),
            MediaEntry::File(download) => {
                let ct = download.content_type.as_deref();
                if is_image_content_type(ct) && download.bytes.len() <= MAX_INLINE_BYTES {
                    let data = base64::engine::general_purpose::STANDARD.encode(&download.bytes);
                    Ok(CallToolResult::success(vec![Content::image(
                        data,
                        ct.unwrap_or("application/octet-stream").to_string(),
                    )]))
                } else if is_text_content_type(ct) {
                    let truncated = download.bytes.len() > MAX_TEXT_BYTES;
                    let slice = &download.bytes[..download.bytes.len().min(MAX_TEXT_BYTES)];
                    let mut text = String::from_utf8_lossy(slice).into_owned();
                    if truncated {
                        text.push_str("\n\n…(truncated at 100 KB)");
                    }
                    Ok(CallToolResult::success(vec![Content::text(text)]))
                } else {
                    Ok(output::json_result(&json!({
                        "path": p.path,
                        "content_type": ct,
                        "size_bytes": download.bytes.len(),
                        "note": "file not returned inline (not text/image, or exceeds 2 MB) — use the CLI `media get` command to download it",
                    })))
                }
            }
        }
    }

    #[rmcp::tool(
        description = "Upload a file to the media store. `path` is the full destination path including filename. `encoding` is \"text\" for plain-text content, or \"base64\" for binary files."
    )]
    pub async fn upload_media(
        &self,
        Parameters(p): Parameters<UploadMediaParams>,
    ) -> Result<CallToolResult, McpError> {
        let bytes = match p.encoding {
            MediaEncoding::Text => p.content.into_bytes(),
            MediaEncoding::Base64 => base64::engine::general_purpose::STANDARD
                .decode(&p.content)
                .map_err(|e| {
                    McpError::invalid_params(format!("invalid base64 content: {e}"), None)
                })?,
        };
        let (dir, filename) = match p.path.rsplit_once('/') {
            Some((dir, filename)) => (dir, filename),
            None => ("", p.path.as_str()),
        };
        if filename.is_empty() {
            return Err(McpError::invalid_params(
                "path must include a filename",
                None,
            ));
        }
        self.client
            .media_upload(dir, filename, bytes)
            .await
            .map_err(|e| err_ctx(&format!("upload to '{}'", p.path), e))?;
        Ok(output::json_result(
            &json!({"path": p.path, "status": "uploaded"}),
        ))
    }

    #[rmcp::tool(
        description = "Create a new folder in the media store. The folder path is relative (e.g. 'images/2025')."
    )]
    pub async fn create_media_folder(
        &self,
        Parameters(p): Parameters<CreateFolderParams>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .media_mkdir(&p.folder)
            .await
            .map_err(|e| err_ctx(&format!("folder '{}'", p.folder), e))?;
        Ok(output::json_result(
            &json!({"folder": p.folder, "status": "created"}),
        ))
    }

    #[rmcp::tool(
        description = "Rename a file or folder in the media store. `new_name` is just the new filename/folder name, not a full path."
    )]
    pub async fn rename_media(
        &self,
        Parameters(p): Parameters<RenameMediaParams>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .media_rename(&p.old_location, &p.new_name)
            .await
            .map_err(|e| err_ctx(&format!("'{}'", p.old_location), e))?;
        Ok(output::json_result(&json!({
            "old_location": p.old_location,
            "new_name": p.new_name,
            "status": "renamed",
        })))
    }

    #[rmcp::tool(
        description = "Delete a file or directory from the media store. Directories must be empty before deletion."
    )]
    pub async fn delete_media(
        &self,
        Parameters(p): Parameters<DeleteMediaParams>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .media_delete(&p.path)
            .await
            .map_err(|e| err_ctx(&format!("media at '{}'", p.path), e))?;
        Ok(output::json_result(
            &json!({"path": p.path, "status": "deleted"}),
        ))
    }
}
