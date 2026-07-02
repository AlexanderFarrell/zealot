use anyhow::{Context, Result, bail};
use zealot_client::types::MediaEntry;

use crate::cli::MediaCmd;
use crate::context::Ctx;
use crate::output::{bold, dim, green, print_json, table::Table};

pub async fn run(ctx: &Ctx, cmd: MediaCmd) -> Result<()> {
    match cmd {
        MediaCmd::Ls { path } => {
            let path = path.unwrap_or_default();
            let files = ctx.client.media_list(&path).await?;
            if ctx.json {
                return print_json(&files);
            }
            if files.is_empty() {
                println!("{}", dim("Empty directory."));
                return Ok(());
            }
            let mut table = Table::new(&["", "Size", "Modified", "Path"]);
            for file in &files {
                let modified = chrono::DateTime::from_timestamp(file.modified_at, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_default();
                table.row(vec![
                    if file.is_folder { "📁".into() } else { "  ".into() },
                    if file.is_folder {
                        String::new()
                    } else {
                        human_size(file.size)
                    },
                    dim(&modified),
                    bold(&file.path),
                ]);
            }
            table.print();
            Ok(())
        }
        MediaCmd::Get { path, out } => {
            match ctx.client.media_get(&path).await? {
                MediaEntry::Directory(_) => bail!("'{path}' is a directory"),
                MediaEntry::File(download) => {
                    let filename = out.unwrap_or_else(|| {
                        path.rsplit('/').next().unwrap_or(&path).to_string()
                    });
                    std::fs::write(&filename, &download.bytes)
                        .with_context(|| format!("failed to write {filename}"))?;
                    println!(
                        "{} Downloaded {} ({})",
                        green("✔"),
                        bold(&filename),
                        human_size(download.bytes.len() as i64)
                    );
                    Ok(())
                }
            }
        }
        MediaCmd::Put { file, path } => {
            let bytes =
                std::fs::read(&file).with_context(|| format!("failed to read {file}"))?;
            let filename = std::path::Path::new(&file)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .context("invalid file name")?;
            let dir = path.unwrap_or_default();
            let size = bytes.len() as i64;
            ctx.client.media_upload(&dir, &filename, bytes).await?;
            let dest = if dir.is_empty() {
                filename.clone()
            } else {
                format!("{dir}/{filename}")
            };
            println!(
                "{} Uploaded {} → {} ({})",
                green("✔"),
                file,
                bold(&dest),
                human_size(size)
            );
            Ok(())
        }
        MediaCmd::Rm { path } => {
            ctx.client.media_delete(&path).await?;
            println!("{} Deleted {}", green("✔"), bold(&path));
            Ok(())
        }
        MediaCmd::Mkdir { path } => {
            ctx.client.media_mkdir(&path).await?;
            println!("{} Created folder {}", green("✔"), bold(&path));
            Ok(())
        }
        MediaCmd::Mv { from, to } => {
            ctx.client.media_rename(&from, &to).await?;
            println!("{} Renamed {} → {}", green("✔"), from, bold(&to));
            Ok(())
        }
    }
}

fn human_size(bytes: i64) -> String {
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64 / 1024.0;
    for unit in ["KB", "MB", "GB", "TB"] {
        if value < 1024.0 {
            return format!("{value:.1} {unit}");
        }
        value /= 1024.0;
    }
    format!("{value:.1} PB")
}
