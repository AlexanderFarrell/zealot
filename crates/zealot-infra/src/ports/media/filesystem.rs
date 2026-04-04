use std::{
    fs,
    io,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use zealot_app::{
    config::ZealotConfig,
    ports::{
        common::PortError,
        media::{MediaError, MediaPort},
    },
};
use zealot_domain::{
    account::Account,
    media::{File, FileStat},
};

#[derive(Debug)]
pub struct MediaFilesystemPort {
    root: PathBuf,
}

impl MediaFilesystemPort {
    pub fn new(config: &ZealotConfig) -> Self {
        Self {
            root: PathBuf::from(&config.media_path),
        }
    }

    /// Returns the root directory for a given account's files.
    fn user_root(&self, account: &Account) -> PathBuf {
        self.root.join(&account.username)
    }

    /// Resolves a relative path into an absolute filesystem path inside the
    /// user's root directory. Performs a secondary bounds check after joining
    /// (defense-in-depth on top of domain-layer sanitization).
    fn resolve_path(&self, rel: &Path, account: &Account) -> Result<PathBuf, PortError<MediaError>> {
        let root = self.user_root(account);
        let full = root.join(rel);

        // Normalize by stripping redundant components without hitting the
        // filesystem (canonicalize would require the path to exist).
        // We use `starts_with` after joining, which is safe here because
        // the domain layer has already stripped `..` and absolute components.
        if !full.starts_with(&root) {
            return Err(io_port_err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "path escapes user root",
            )));
        }

        Ok(full)
    }
}

// ─── Error helpers ────────────────────────────────────────────────────────────

fn io_port_err(err: io::Error) -> PortError<MediaError> {
    PortError::OtherError {
        err: MediaError::IoError { err },
    }
}

fn not_found_err() -> PortError<MediaError> {
    io_port_err(io::Error::new(io::ErrorKind::NotFound, "not found"))
}

// ─── ETag ─────────────────────────────────────────────────────────────────────

/// Computes a SHA-256 hex digest of the file at `path`.
pub fn calculate_etag(full_path: &Path) -> Result<String, io::Error> {
    let bytes = fs::read(full_path)?;
    let hash = Sha256::digest(&bytes);
    Ok(hex::encode(hash))
}

// ─── MediaPort implementation ─────────────────────────────────────────────────

impl MediaPort for MediaFilesystemPort {
    /// Returns the user's root directory path, creating it if it does not exist.
    fn user_path(&self, account: &Account) -> Result<PathBuf, PortError<MediaError>> {
        let root = self.user_root(account);
        fs::create_dir_all(&root).map_err(io_port_err)?;
        Ok(root)
    }

    /// Lists directory entries (if `path` is a directory) or returns a
    /// single-element vec with the file's stat (if `path` is a file).
    fn get(&self, path: &Path, account: &Account) -> Result<Vec<FileStat>, PortError<MediaError>> {
        let full = self.resolve_path(path, account)?;

        let metadata = fs::metadata(&full).map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                not_found_err()
            } else {
                io_port_err(e)
            }
        })?;

        if metadata.is_dir() {
            let entries = fs::read_dir(&full).map_err(io_port_err)?;
            let mut stats = Vec::new();
            for entry in entries {
                let entry = entry.map_err(io_port_err)?;
                let entry_meta = entry.metadata().map_err(io_port_err)?;
                let entry_name = entry.file_name();
                let relative = path.join(entry_name);
                let stat = FileStat::from_metadata(&relative, &entry_meta)
                    .map_err(|e| io_port_err(io::Error::other(e.to_string())))?;
                stats.push(stat);
            }
            Ok(stats)
        } else {
            let stat = FileStat::from_metadata(path, &metadata)
                .map_err(|e| io_port_err(io::Error::other(e.to_string())))?;
            Ok(vec![stat])
        }
    }

    /// Downloads a file: reads its bytes and returns a `File` with stat and
    /// contents. Returns `None` if the path does not exist.
    fn download(&self, path: &Path, account: &Account) -> Result<Option<File>, PortError<MediaError>> {
        let full = self.resolve_path(path, account)?;

        let metadata = match fs::metadata(&full) {
            Ok(m) => m,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(io_port_err(e)),
        };

        if metadata.is_dir() {
            return Ok(None);
        }

        let bytes = fs::read(&full).map_err(io_port_err)?;
        let stat = FileStat::from_metadata(path, &metadata)
            .map_err(|e| io_port_err(io::Error::other(e.to_string())))?;

        Ok(Some(File { stat, bytes }))
    }

    /// Writes `bytes` to `path` (which is the full destination file path,
    /// including filename). Returns the `FileStat` of the newly written file.
    fn upload(
        &self,
        path: &Path,
        bytes: &Vec<u8>,
        account: &Account,
    ) -> Result<Option<FileStat>, PortError<MediaError>> {
        let full = self.resolve_path(path, account)?;

        // Ensure the parent directory exists.
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).map_err(io_port_err)?;
        }

        fs::write(&full, bytes).map_err(io_port_err)?;

        let metadata = fs::metadata(&full).map_err(io_port_err)?;
        let stat = FileStat::from_metadata(path, &metadata)
            .map_err(|e| io_port_err(io::Error::other(e.to_string())))?;

        Ok(Some(stat))
    }

    /// Creates a directory (and any intermediate directories) at `path`.
    fn make_folder(&self, path: &Path, account: &Account) -> Result<(), PortError<MediaError>> {
        let full = self.resolve_path(path, account)?;
        fs::create_dir_all(&full).map_err(io_port_err)
    }

    /// Renames the file or directory at `path` to `new_path`.
    /// The service layer guarantees `new_path` stays in the same parent directory.
    fn rename(
        &self,
        path: &Path,
        new_path: &Path,
        account: &Account,
    ) -> Result<(), PortError<MediaError>> {
        let full_old = self.resolve_path(path, account)?;
        let full_new = self.resolve_path(new_path, account)?;
        fs::rename(&full_old, &full_new).map_err(io_port_err)
    }

    /// Deletes the file or directory (recursively) at `path`.
    fn delete(&self, path: &Path, account: &Account) -> Result<(), PortError<MediaError>> {
        let full = self.resolve_path(path, account)?;

        let metadata = match fs::metadata(&full) {
            Ok(m) => m,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Err(not_found_err()),
            Err(e) => return Err(io_port_err(e)),
        };

        if metadata.is_dir() {
            fs::remove_dir_all(&full).map_err(io_port_err)
        } else {
            fs::remove_file(&full).map_err(io_port_err)
        }
    }

    /// Deletes a directory and all of its contents.
    fn delete_folder(&self, path: &Path, account: &Account) -> Result<(), PortError<MediaError>> {
        let full = self.resolve_path(path, account)?;
        fs::remove_dir_all(&full).map_err(io_port_err)
    }

    /// Returns `true` if the path exists within the user's root directory.
    fn exists(&self, path: &Path, account: &Account) -> Result<bool, PortError<MediaError>> {
        let full = self.resolve_path(path, account)?;
        Ok(full.exists())
    }
}
