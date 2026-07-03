//! $EDITOR round-trip (same contract as the CLI's editor module).

use std::io::Write;
use std::process::Command;

use anyhow::{Context, Result, bail};

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

    let mut parts = editor.split_whitespace();
    let program = parts.next().unwrap_or("vi");
    let status = Command::new(program)
        .args(parts)
        .arg(&path)
        .status()
        .with_context(|| format!("failed to launch editor '{editor}'"))?;
    if !status.success() {
        bail!("editor exited with {status}");
    }

    let edited = std::fs::read_to_string(&path).context("failed to read edited file")?;
    if edited == initial {
        Ok(None)
    } else {
        Ok(Some(edited))
    }
}
