use serde_json::json;
use zealot_domain::media::FileStatDto;

use crate::{
    ApiError, ZealotClient,
    api::media_path,
    types::{DirectoryResponse, MediaEntry},
};

impl ZealotClient {
    /// `GET /media/{path}` — a directory listing or a file download, depending
    /// on what the path points at. Root listing when `path` is empty.
    pub async fn media_get(&self, path: &str) -> Result<MediaEntry, ApiError> {
        let encoded = media_path(path);
        let url = if encoded.is_empty() {
            "/media/".to_string()
        } else {
            format!("/media/{encoded}")
        };
        let download = self.get_bytes(&url).await?;
        let is_json = download
            .content_type
            .as_deref()
            .is_some_and(|ct| ct.starts_with("application/json"));
        if is_json {
            // Directory listings are {"files": [...]}. A stored .json file also
            // arrives as application/json, so fall through on parse failure.
            if let Ok(dir) = serde_json::from_slice::<DirectoryResponse>(&download.bytes) {
                return Ok(MediaEntry::Directory(dir.files));
            }
        }
        Ok(MediaEntry::File(download))
    }

    /// Convenience: list a directory, erroring if the path is a file.
    pub async fn media_list(&self, path: &str) -> Result<Vec<FileStatDto>, ApiError> {
        match self.media_get(path).await? {
            MediaEntry::Directory(files) => Ok(files),
            MediaEntry::File(_) => Err(ApiError::Http {
                status: reqwest::StatusCode::BAD_REQUEST,
                message: format!("'{path}' is a file, not a directory"),
            }),
        }
    }

    /// Upload a file into directory `dir` (multipart, field carries filename).
    pub async fn media_upload(
        &self,
        dir: &str,
        filename: &str,
        bytes: Vec<u8>,
    ) -> Result<(), ApiError> {
        let encoded = media_path(dir);
        // The wildcard route needs a non-empty path; uploads to the root use
        // the filename itself as the path segment.
        let url = if encoded.is_empty() {
            format!("/media/{}", media_path(filename))
        } else {
            format!("/media/{encoded}")
        };
        self.post_multipart(&url, "file", filename, bytes).await
    }

    pub async fn media_delete(&self, path: &str) -> Result<(), ApiError> {
        self.delete(&format!("/media/{}", media_path(path))).await
    }

    pub async fn media_mkdir(&self, folder: &str) -> Result<(), ApiError> {
        self.post_no_response("/media/mkdir", &json!({ "folder": folder }))
            .await
    }

    pub async fn media_rename(&self, old_location: &str, new_name: &str) -> Result<(), ApiError> {
        self.patch_no_response(
            "/media/rename",
            &json!({ "old_location": old_location, "new_name": new_name }),
        )
        .await
    }
}
