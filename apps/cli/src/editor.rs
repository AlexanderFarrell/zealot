//! `$EDITOR` helpers, including durable save-on-write item drafts.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use tokio::process::Command as TokioCommand;
use tokio::time::{Instant, timeout};
use zealot_client::ZealotClient;
use zealot_domain::item::UpdateItemDto;

const POLL_INTERVAL: Duration = Duration::from_millis(250);
const DEBOUNCE: Duration = Duration::from_millis(300);
const FINAL_SAVE_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_RETRY: Duration = Duration::from_secs(30);

/// Result of a durable item edit session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemEditOutcome {
    Saved,
    NoChanges,
    ReadOnly,
}

/// Open `initial` in the user's editor and return the saved text, or `None`
/// when nothing changed. `extension` controls syntax highlighting (e.g. "md").
pub fn edit_text(initial: &str, extension: &str) -> Result<Option<String>> {
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string());

    let mut file = tempfile::Builder::new()
        .prefix("zealot-")
        .suffix(&format!(".{extension}"))
        .tempfile()
        .context("failed to create temp file")?;
    file.write_all(initial.as_bytes())?;
    file.flush()?;
    let path = file.path().to_path_buf();

    // Support EDITOR values with arguments ("code --wait").
    let mut parts = editor.split_whitespace();
    let program = parts.next().unwrap_or("vi");
    let status = Command::new(program)
        .args(parts)
        .arg(&path)
        .status()
        .with_context(|| format!("failed to launch editor '{editor}'"))?;
    if !status.success() {
        bail!("editor exited with {status}; aborting");
    }

    let edited = std::fs::read_to_string(&path).context("failed to read edited file")?;
    if edited == initial {
        Ok(None)
    } else {
        Ok(Some(edited))
    }
}

/// Whether a numeric item has an unsynced draft for this connection identity.
pub fn has_item_draft(identity: &str, item_id: i64) -> Result<bool> {
    Ok(draft_paths(identity, item_id)?.draft.exists())
}

/// Edit an item's content from a durable local draft, synchronising each settled
/// file write while the editor is open. `initial` is only needed for a new draft.
pub async fn edit_item_content(
    identity: &str,
    item_id: i64,
    initial: Option<&str>,
    client: ZealotClient,
) -> Result<ItemEditOutcome> {
    let paths = draft_paths(identity, item_id)?;
    fs::create_dir_all(&paths.dir).context("failed to create item draft directory")?;

    let lock = match DraftLock::acquire(&paths.lock)? {
        Some(lock) => lock,
        None => return open_read_only_snapshot(&paths).await,
    };

    let resumed = paths.draft.exists();
    let initial = match (resumed, initial) {
        (true, _) => fs::read_to_string(&paths.draft).context("failed to read item draft")?,
        (false, Some(content)) => create_draft(&paths.draft, content)?,
        (false, None) => bail!(
            "cannot recover item #{item_id}: no local draft exists and the item could not be loaded"
        ),
    };

    let result = run_item_editor(&paths.draft, item_id, initial, resumed, client).await;
    match result {
        Ok(outcome) => {
            // A draft is deleted only after the latest revision is acknowledged.
            fs::remove_file(&paths.draft).context("failed to remove acknowledged item draft")?;
            drop(lock);
            Ok(outcome)
        }
        Err(error) => {
            drop(lock);
            Err(error)
        }
    }
}

async fn run_item_editor(
    draft: &Path,
    item_id: i64,
    initial: String,
    resumed: bool,
    client: ZealotClient,
) -> Result<ItemEditOutcome> {
    let mut editor = launch_async_editor(draft)?;
    let mut observed = initial.clone();
    let mut pending = resumed.then_some(initial);
    let mut changed_at = Instant::now() - DEBOUNCE;
    let mut retry_at = Instant::now();
    let mut retry_delay = Duration::from_secs(1);
    let mut changed = resumed;

    loop {
        tokio::time::sleep(POLL_INTERVAL).await;
        let current = fs::read_to_string(draft).context("failed to read item draft")?;
        if current != observed {
            observed = current.clone();
            pending = Some(current);
            changed_at = Instant::now();
            retry_at = changed_at + DEBOUNCE;
            retry_delay = Duration::from_secs(1);
            changed = true;
        }

        if let Some(content) = pending.clone()
            && Instant::now() >= changed_at + DEBOUNCE
            && Instant::now() >= retry_at
        {
            match save_content(&client, item_id, content).await {
                Ok(()) => {
                    pending = None;
                    retry_delay = Duration::from_secs(1);
                }
                Err(_) => {
                    retry_at = Instant::now() + retry_delay;
                    retry_delay = (retry_delay * 2).min(MAX_RETRY);
                }
            }
        }

        if let Some(status) = editor.try_wait().context("failed to wait for editor")? {
            if !status.success() {
                bail!(
                    "editor exited with {status}; draft retained at {}",
                    draft.display()
                );
            }
            break;
        }
    }

    let final_content = fs::read_to_string(draft).context("failed to read final item draft")?;
    if final_content != observed {
        pending = Some(final_content);
        changed = true;
    }
    if let Some(content) = pending {
        timeout(FINAL_SAVE_TIMEOUT, save_content(&client, item_id, content))
            .await
            .map_err(|_| {
                anyhow::anyhow!(
                    "could not save item #{item_id}; draft retained at {}",
                    draft.display()
                )
            })?
            .map_err(|_| {
                anyhow::anyhow!(
                    "could not save item #{item_id}; draft retained at {}",
                    draft.display()
                )
            })?;
    }

    Ok(if changed {
        ItemEditOutcome::Saved
    } else {
        ItemEditOutcome::NoChanges
    })
}

async fn save_content(client: &ZealotClient, item_id: i64, content: String) -> Result<()> {
    let dto = UpdateItemDto {
        item_id,
        title: None,
        content: Some(content),
        attributes: None,
        links: None,
    };
    client.update_item(&dto).await.context("item save failed")?;
    Ok(())
}

fn launch_async_editor(path: &Path) -> Result<tokio::process::Child> {
    let editor = editor_command();
    let mut parts = editor.split_whitespace();
    let program = parts.next().unwrap_or("vi");
    TokioCommand::new(program)
        .args(parts)
        .arg(path)
        .spawn()
        .with_context(|| format!("failed to launch editor '{editor}'"))
}

fn editor_command() -> String {
    std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string())
}

async fn open_read_only_snapshot(paths: &DraftPaths) -> Result<ItemEditOutcome> {
    // The owning session creates its draft immediately after taking the lock.
    // Allow that small handoff window before declaring the lock inconsistent.
    let mut content = None;
    for _ in 0..20 {
        match fs::read_to_string(&paths.draft) {
            Ok(text) => {
                content = Some(text);
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Err(error) => return Err(error).context("failed to read active item draft"),
        }
    }
    let content = content.with_context(|| {
        format!(
            "item is already being edited, but its draft is unavailable: {}",
            paths.draft.display()
        )
    })?;
    let mut snapshot = tempfile::Builder::new()
        .prefix("zealot-read-only-")
        .suffix(".md")
        .tempfile()
        .context("failed to create read-only editor snapshot")?;
    snapshot.write_all(content.as_bytes())?;
    snapshot.flush()?;
    let path = snapshot.path().to_path_buf();
    let mut editor = launch_async_editor(&path)?;
    let status = editor.wait().await.context("failed to wait for editor")?;
    if !status.success() {
        bail!("editor exited with {status}; read-only snapshot discarded");
    }
    Ok(ItemEditOutcome::ReadOnly)
}

fn create_draft(path: &Path, content: &str) -> Result<String> {
    let parent = path
        .parent()
        .context("item draft has no parent directory")?;
    let mut temp =
        tempfile::NamedTempFile::new_in(parent).context("failed to create item draft")?;
    temp.write_all(content.as_bytes())?;
    temp.as_file().sync_all()?;
    match temp.persist_noclobber(path) {
        Ok(_) => Ok(content.to_string()),
        Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
            fs::read_to_string(path).context("failed to read concurrently-created item draft")
        }
        Err(error) => Err(error.error).context("failed to create item draft"),
    }
}

struct DraftPaths {
    dir: PathBuf,
    draft: PathBuf,
    lock: PathBuf,
}

fn draft_paths(identity: &str, item_id: i64) -> Result<DraftPaths> {
    let root = match std::env::var("XDG_STATE_HOME") {
        Ok(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => PathBuf::from(std::env::var("HOME").context("HOME is required for item drafts")?)
            .join(".local/state"),
    };
    let dir = root.join("zealot/drafts").join(identity);
    let stem = item_id.to_string();
    Ok(DraftPaths {
        draft: dir.join(format!("{stem}.md")),
        lock: dir.join(format!("{stem}.lock")),
        dir,
    })
}

struct DraftLock {
    path: PathBuf,
    contents: String,
}

impl DraftLock {
    fn acquire(path: &Path) -> Result<Option<Self>> {
        let contents = std::process::id().to_string();
        match Self::create(path, &contents) {
            Ok(()) => Ok(Some(Self {
                path: path.into(),
                contents,
            })),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let stale = fs::read_to_string(path)
                    .ok()
                    .and_then(|pid| pid.trim().parse::<u32>().ok())
                    .is_none_or(process_is_dead);
                if !stale {
                    return Ok(None);
                }
                if let Err(error) = fs::remove_file(path)
                    && error.kind() != std::io::ErrorKind::NotFound
                {
                    return Err(error).context("failed to reclaim stale item draft lock");
                }
                Self::create(path, &contents)
                    .context("failed to create item draft lock after reclaiming stale lock")?;
                Ok(Some(Self {
                    path: path.into(),
                    contents,
                }))
            }
            Err(error) => Err(error).context("failed to create item draft lock"),
        }
    }

    fn create(path: &Path, contents: &str) -> std::io::Result<()> {
        let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()
    }
}

impl Drop for DraftLock {
    fn drop(&mut self) {
        if fs::read_to_string(&self.path).ok().as_deref() == Some(&self.contents) {
            let _ = fs::remove_file(&self.path);
        }
    }
}

#[cfg(unix)]
fn process_is_dead(pid: u32) -> bool {
    if pid > i32::MAX as u32 {
        return true;
    }
    unsafe {
        libc::kill(pid as i32, 0) != 0
            && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
    }
}

#[cfg(not(unix))]
fn process_is_dead(_pid: u32) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::{DraftLock, create_draft};
    use std::fs;

    #[test]
    fn draft_creation_is_atomic_and_preserves_an_existing_draft() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("42.md");
        assert_eq!(create_draft(&path, "first").unwrap(), "first");
        assert_eq!(create_draft(&path, "second").unwrap(), "first");
        assert_eq!(fs::read_to_string(path).unwrap(), "first");
    }

    #[test]
    fn a_live_lock_blocks_a_second_writer_and_is_removed_by_its_owner() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("42.lock");
        let lock = DraftLock::acquire(&path).unwrap().unwrap();
        assert!(DraftLock::acquire(&path).unwrap().is_none());
        drop(lock);
        assert!(!path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn a_stale_lock_is_reclaimed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("42.lock");
        fs::write(&path, "4294967295").unwrap();
        let lock = DraftLock::acquire(&path).unwrap().unwrap();
        drop(lock);
        assert!(!path.exists());
    }
}
