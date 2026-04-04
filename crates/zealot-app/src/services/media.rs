use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use zealot_domain::{
    account::Account,
    media::{File, FileStat, MediaError},
};

use crate::ports::{
    common::PortError,
    media::{MediaError as PortMediaError, MediaPort},
};

// ─── Public types ─────────────────────────────────────────────────────────────

/// The result of a `get` call — either a directory listing or a single file.
pub enum MediaEntry {
    Directory(Vec<FileStat>),
    File(File),
}

#[derive(Debug, thiserror::Error)]
pub enum MediaServiceError {
    #[error("not found")]
    NotFound,

    #[error("invalid path: {reason}")]
    InvalidPath { reason: String },

    #[error("port error: {0}")]
    Port(#[from] PortError<PortMediaError>),
}

// ─── Service ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MediaService {
    port: Arc<dyn MediaPort>,
}

impl MediaService {
    pub fn new(port: &Arc<dyn MediaPort>) -> Self {
        Self { port: port.clone() }
    }

    // ─── Queries ──────────────────────────────────────────────────────────────

    /// Returns the contents of a directory, or the file at the given path.
    /// The `rel_path` may be empty to list the account's root directory.
    pub fn get(&self, rel_path: &str, account: &Account) -> Result<MediaEntry, MediaServiceError> {
        // Ensure the user root exists before any operation.
        self.port.user_path(account)?;

        if rel_path.is_empty() {
            // List the user's root directory.
            let root = PathBuf::new();
            let stats = self.port.get(&root, account)?;
            return Ok(MediaEntry::Directory(stats));
        }

        let path = validated_path(rel_path)?;

        // Try to download as a file first. If `None` is returned, it is a directory.
        match self.port.download(&path, account)? {
            Some(file) => Ok(MediaEntry::File(file)),
            None => {
                let stats = self.port.get(&path, account)?;
                Ok(MediaEntry::Directory(stats))
            }
        }
    }

    // ─── Mutations ────────────────────────────────────────────────────────────

    /// Uploads `bytes` as a file named `filename` inside `dir_path`.
    /// `filename` is sanitized to its base component only (no directory traversal).
    pub fn upload(
        &self,
        dir_path: &str,
        filename: &str,
        bytes: Vec<u8>,
        account: &Account,
    ) -> Result<FileStat, MediaServiceError> {
        let safe_name = safe_filename(filename)?;

        let dest_path = if dir_path.is_empty() {
            PathBuf::from(&safe_name)
        } else {
            let dir = validated_path(dir_path)?;
            dir.join(&safe_name)
        };

        match self.port.upload(&dest_path, &bytes, account)? {
            Some(stat) => Ok(stat),
            None => Err(MediaServiceError::NotFound),
        }
    }

    /// Creates a folder (and any intermediate directories) at `folder_path`.
    pub fn make_folder(&self, folder_path: &str, account: &Account) -> Result<(), MediaServiceError> {
        if folder_path.is_empty() {
            return Err(MediaServiceError::InvalidPath {
                reason: String::from("folder path must not be empty"),
            });
        }
        if folder_path.len() > 1024 {
            return Err(MediaServiceError::InvalidPath {
                reason: String::from("path is too long"),
            });
        }
        let path = validated_path(folder_path)?;
        self.port.make_folder(&path, account)?;
        Ok(())
    }

    /// Renames the file or directory at `old_location` to `new_name`.
    /// `new_name` must be a plain filename (no path separators) — the renamed
    /// entry stays in the same parent directory.
    pub fn rename(
        &self,
        old_location: &str,
        new_name: &str,
        account: &Account,
    ) -> Result<(), MediaServiceError> {
        if old_location.is_empty() || new_name.is_empty() {
            return Err(MediaServiceError::InvalidPath {
                reason: String::from("old_location and new_name must not be empty"),
            });
        }

        let old_path = validated_path(old_location)?;

        // `new_name` must be a bare filename — reject any separators.
        let clean_name = Path::new(new_name)
            .file_name()
            .and_then(|n| n.to_str())
            .map(String::from)
            .filter(|n| !n.is_empty())
            .ok_or_else(|| MediaServiceError::InvalidPath {
                reason: String::from("new_name must be a plain filename with no path separators"),
            })?;

        // Keep the renamed entry in the same directory as the original.
        let new_path = match old_path.parent() {
            Some(parent) if parent != Path::new("") => parent.join(&clean_name),
            _ => PathBuf::from(&clean_name),
        };

        self.port.rename(&old_path, &new_path, account)?;
        Ok(())
    }

    /// Deletes the file or directory at `rel_path`. An empty path is rejected
    /// to prevent accidental deletion of the account's entire media root.
    pub fn delete(&self, rel_path: &str, account: &Account) -> Result<(), MediaServiceError> {
        if rel_path.is_empty() {
            return Err(MediaServiceError::InvalidPath {
                reason: String::from("path must not be empty — cannot delete the root directory"),
            });
        }
        let path = validated_path(rel_path)?;
        self.port.delete(&path, account)?;
        Ok(())
    }
}

// ─── Path helpers ─────────────────────────────────────────────────────────────

/// Validates and sanitizes a relative path string. Rejects null bytes and paths
/// that are too long. The domain layer's `FileStat::new` (via
/// `sanitize_relative_path`) enforces the remaining rules (`..`, absolute paths).
fn validated_path(raw: &str) -> Result<PathBuf, MediaServiceError> {
    if raw.contains('\x00') {
        return Err(MediaServiceError::InvalidPath {
            reason: String::from("path contains null bytes"),
        });
    }
    if raw.len() > 1024 {
        return Err(MediaServiceError::InvalidPath {
            reason: String::from("path is too long"),
        });
    }

    // Leverage the domain's sanitize logic by constructing a FileStat, which
    // runs `sanitize_relative_path` internally and rejects `..`, absolute paths,
    // and empty paths.
    use chrono::Utc;
    use size::Size;
    FileStat::new(raw, Size::from_bytes(0), false, Utc::now().naive_utc())
        .map(|stat| stat.path().to_path_buf())
        .map_err(|e: MediaError| MediaServiceError::InvalidPath {
            reason: e.to_string(),
        })
}

/// Extracts and validates the base filename component, rejecting anything that
/// would allow directory traversal via the filename field of a multipart upload.
fn safe_filename(filename: &str) -> Result<String, MediaServiceError> {
    if filename.is_empty() {
        return Err(MediaServiceError::InvalidPath {
            reason: String::from("filename must not be empty"),
        });
    }
    let base = Path::new(filename)
        .file_name()
        .and_then(|n| n.to_str())
        .map(String::from)
        .filter(|n| !n.is_empty())
        .ok_or_else(|| MediaServiceError::InvalidPath {
            reason: String::from("filename is invalid"),
        })?;
    Ok(base)
}
