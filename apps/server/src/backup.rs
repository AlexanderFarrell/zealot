//! Operator-facing, versioned backup bundles.  This module intentionally has no HTTP surface.
use chrono::{SecondsFormat, Utc};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    process::Command,
};
use uuid::Uuid;
use zealot_app::config::ZealotConfig;

const FORMAT: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub format_version: u32,
    pub created_at: String,
    pub zealot_version: String,
    pub database_engine: String,
    pub dump_format: String,
    pub migrations: Vec<String>,
    pub table_counts: BTreeMap<String, i64>,
    pub media_files: usize,
    pub media_bytes: u64,
    pub artifacts: BTreeMap<String, String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub format_version: u32,
    pub time: String,
    pub reason: String,
    pub outcome: String,
    pub bundle: Option<String>,
    pub verification: Option<bool>,
    pub error: Option<String>,
    pub pruned: Vec<String>,
}
#[derive(Debug, Serialize)]
pub struct Status {
    pub latest: Option<Receipt>,
    pub bundles: Vec<String>,
}

struct Lock(PathBuf);
impl Lock {
    fn acquire(dir: &Path) -> Result<Self, String> {
        fs::create_dir_all(dir).map_err(io)?;
        let p = dir.join(".backup.lock");
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&p)
            .map_err(|_| "a backup or restore is already running".to_string())?;
        Ok(Self(p))
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
fn io(e: std::io::Error) -> String {
    e.to_string()
}
fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}
fn sha(path: &Path) -> Result<String, String> {
    let mut f = File::open(path).map_err(io)?;
    let mut h = Sha256::new();
    let mut b = [0u8; 32768];
    loop {
        let n = f.read(&mut b).map_err(io)?;
        if n == 0 {
            break;
        };
        h.update(&b[..n]);
    }
    Ok(hex::encode(h.finalize()))
}
fn copy_tree(
    from: &Path,
    to: &Path,
    hashes: &mut BTreeMap<String, String>,
    bytes: &mut u64,
    files: &mut usize,
) -> Result<(), String> {
    if !from.exists() {
        return Ok(());
    }
    fs::create_dir_all(to).map_err(io)?;
    for e in fs::read_dir(from).map_err(io)? {
        let e = e.map_err(io)?;
        let rel = e.file_name();
        let dest = to.join(&rel);
        let ft = e.file_type().map_err(io)?;
        if ft.is_dir() {
            copy_tree(&e.path(), &dest, hashes, bytes, files)?;
        } else if ft.is_file() {
            fs::copy(e.path(), &dest).map_err(io)?;
            let len = e.metadata().map_err(io)?.len();
            *bytes += len;
            *files += 1;
            hashes.insert(
                format!("media/{}", dest.strip_prefix(to).unwrap_or(&dest).display()),
                sha(&dest)?,
            );
        }
    }
    Ok(())
}
fn gather_media(
    root: &Path,
    dir: &Path,
    hashes: &mut BTreeMap<String, String>,
    bytes: &mut u64,
    files: &mut usize,
) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    for e in fs::read_dir(dir).map_err(io)? {
        let e = e.map_err(io)?;
        if e.file_type().map_err(io)?.is_dir() {
            gather_media(root, &e.path(), hashes, bytes, files)?
        } else if e.file_type().map_err(io)?.is_file() {
            let p = e.path();
            *bytes += e.metadata().map_err(io)?.len();
            *files += 1;
            hashes.insert(
                format!(
                    "media/{}",
                    p.strip_prefix(root)
                        .map_err(|_| "invalid media path")?
                        .display()
                ),
                sha(&p)?,
            );
        }
    }
    Ok(())
}
fn sqlite_counts(db: &Path) -> Result<(Vec<String>, BTreeMap<String, i64>), String> {
    let out = Command::new("sqlite3")
        .arg(db)
        .arg("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%';")
        .output()
        .map_err(|_| "sqlite3 is required for backup verification".to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }
    let names = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let mut counts = BTreeMap::new();
    for n in &names {
        let q = format!("SELECT count(*) FROM \"{}\";", n.replace('"', "\"\""));
        let o = Command::new("sqlite3")
            .arg(db)
            .arg(q)
            .output()
            .map_err(io)?;
        counts.insert(
            n.clone(),
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .parse()
                .unwrap_or(-1),
        );
    }
    let migrations = Command::new("sqlite3")
        .arg(db)
        .arg("SELECT version FROM _sqlx_migrations ORDER BY version;")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    Ok((migrations, counts))
}
fn pg_args(c: &ZealotConfig) -> Vec<String> {
    let mut a = vec![
        "--format=custom".into(),
        "--no-owner".into(),
        "--no-acl".into(),
        "--host".into(),
        c.db_host.clone().unwrap_or_else(|| "localhost".into()),
    ];
    if let Some(u) = &c.db_username {
        a.extend(["--username".into(), u.clone()])
    }
    if let Some(d) = &c.db_database {
        a.push(d.clone())
    }
    a
}
fn write_receipt(dir: &Path, r: &Receipt) -> Result<(), String> {
    let line = serde_json::to_string(r).map_err(|e| e.to_string())? + "\n";
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("attempts.jsonl"))
        .map_err(io)?;
    f.write_all(line.as_bytes()).map_err(io)?;
    fs::write(
        dir.join("latest-status.json"),
        serde_json::to_vec_pretty(r).map_err(|e| e.to_string())?,
    )
    .map_err(io)
}

pub fn create(
    config: &ZealotConfig,
    destination: Option<&Path>,
    reason: &str,
) -> Result<PathBuf, String> {
    let dir = destination.unwrap_or_else(|| Path::new(&config.backup_path));
    let _lock = Lock::acquire(dir)?;
    let result = create_locked(config, dir);
    let receipt = match &result {
        Ok(p) => Receipt {
            format_version: FORMAT,
            time: now(),
            reason: reason.into(),
            outcome: "success".into(),
            bundle: Some(p.display().to_string()),
            verification: Some(true),
            error: None,
            pruned: Vec::new(),
        },
        Err(e) => Receipt {
            format_version: FORMAT,
            time: now(),
            reason: reason.into(),
            outcome: "failure".into(),
            bundle: None,
            verification: None,
            error: Some(e.clone()),
            pruned: Vec::new(),
        },
    };
    let _ = write_receipt(dir, &receipt);
    result
}
fn create_locked(c: &ZealotConfig, dir: &Path) -> Result<PathBuf, String> {
    let staging = dir.join(format!(".backup-{}", Uuid::new_v4()));
    fs::create_dir_all(&staging).map_err(io)?;
    let work = staging.join("contents");
    fs::create_dir_all(work.join("media")).map_err(io)?;
    let mut artifacts = BTreeMap::new();
    let dump = if c.database == "sqlite" {
        if !Path::new(&c.db_filename).exists() {
            let _ = fs::remove_dir_all(&staging);
            return Err("no SQLite database exists yet".into());
        }
        let db = work.join("database.sqlite");
        let escaped = db.display().to_string().replace('\'', "''");
        let out = Command::new("sqlite3")
            .arg(&c.db_filename)
            .arg(format!("VACUUM INTO '{}';", escaped))
            .output()
            .map_err(|_| "sqlite3 is required for online SQLite backups".to_string())?;
        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).to_string());
        }
        artifacts.insert("database.sqlite".into(), sha(&db)?);
        "sqlite".into()
    } else if c.database == "postgres" {
        let db = work.join("database.dump");
        let mut cmd = Command::new("pg_dump");
        cmd.args(pg_args(c)).arg("--file").arg(&db);
        if let Some(p) = &c.db_password {
            cmd.env("PGPASSWORD", p);
        }
        let out = cmd
            .output()
            .map_err(|_| "pg_dump is required for PostgreSQL backups".to_string())?;
        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).to_string());
        }
        artifacts.insert("database.dump".into(), sha(&db)?);
        "postgres-custom".into()
    } else {
        return Err(format!("unsupported database engine {}", c.database));
    };
    let mut media_bytes = 0;
    let mut media_files = 0;
    gather_media(
        Path::new(&c.media_path),
        Path::new(&c.media_path),
        &mut artifacts,
        &mut media_bytes,
        &mut media_files,
    )?;
    copy_tree(
        Path::new(&c.media_path),
        &work.join("media"),
        &mut BTreeMap::new(),
        &mut 0,
        &mut 0,
    )?;
    let (migrations, table_counts) = if c.database == "sqlite" {
        sqlite_counts(&work.join("database.sqlite"))?
    } else {
        (Vec::new(), BTreeMap::new())
    };
    let manifest = Manifest {
        format_version: FORMAT,
        created_at: now(),
        zealot_version: env!("CARGO_PKG_VERSION").into(),
        database_engine: c.database.clone(),
        dump_format: dump,
        migrations,
        table_counts,
        media_files,
        media_bytes,
        artifacts,
    };
    fs::write(
        work.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(io)?;
    let bundle = dir.join(format!(
        "zealot-backup-v1-{}-{}.tar.gz",
        Utc::now().format("%Y%m%dT%H%M%SZ"),
        Uuid::new_v4()
    ));
    let tmp = bundle.with_extension("tar.gz.tmp");
    let f = File::create(&tmp).map_err(io)?;
    let mut tar = tar::Builder::new(GzEncoder::new(f, Compression::default()));
    tar.append_dir_all(".", &work).map_err(|e| e.to_string())?;
    let encoder = tar.into_inner().map_err(|e| e.to_string())?;
    encoder.finish().map_err(|e| e.to_string())?;
    fs::rename(&tmp, &bundle).map_err(io)?;
    verify(c, &bundle)?;
    retain(dir, c.backup_retention_count)?;
    let _ = fs::remove_dir_all(staging);
    Ok(bundle)
}
pub fn verify(c: &ZealotConfig, bundle: &Path) -> Result<(), String> {
    let stage = std::env::temp_dir().join(format!("zealot-verify-{}", Uuid::new_v4()));
    extract_safe(bundle, &stage)?;
    let raw = fs::read(stage.join("manifest.json")).map_err(io)?;
    let m: Manifest = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
    if m.format_version != FORMAT || m.database_engine != c.database {
        return Err("backup format or database engine is incompatible".into());
    }
    for (name, expect) in &m.artifacts {
        let p = stage.join(name);
        if !p.is_file() || sha(&p)? != *expect {
            return Err(format!("artifact hash mismatch: {name}"));
        }
    }
    if c.database == "sqlite" {
        let o = Command::new("sqlite3")
            .arg(stage.join("database.sqlite"))
            .arg("PRAGMA integrity_check;")
            .output()
            .map_err(|_| "sqlite3 is required for verification".to_string())?;
        if String::from_utf8_lossy(&o.stdout).trim() != "ok" {
            return Err("SQLite integrity check failed".into());
        };
        let (_, counts) = sqlite_counts(&stage.join("database.sqlite"))?;
        if counts != m.table_counts {
            return Err("SQLite table counts differ from manifest".into());
        }
    } else {
        let o = Command::new("pg_restore")
            .arg("--list")
            .arg(stage.join("database.dump"))
            .output()
            .map_err(|_| "pg_restore is required for verification".to_string())?;
        if !o.status.success() {
            return Err("PostgreSQL dump is unreadable".into());
        }
    }
    let _ = fs::remove_dir_all(stage);
    Ok(())
}
fn safe(p: &Path) -> bool {
    !p.is_absolute()
        && !p.components().any(|c| {
            matches!(
                c,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
}
fn extract_safe(bundle: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(io)?;
    let f = File::open(bundle).map_err(io)?;
    let mut ar = tar::Archive::new(GzDecoder::new(f));
    for e in ar.entries().map_err(|e| e.to_string())? {
        let mut e = e.map_err(|e| e.to_string())?;
        let p = e.path().map_err(|e| e.to_string())?.into_owned();
        if !safe(&p) {
            return Err("unsafe archive path".into());
        }
        e.unpack_in(dest).map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn retain(dir: &Path, count: usize) -> Result<(), String> {
    let mut files = fs::read_dir(dir)
        .map_err(io)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.starts_with("zealot-backup-v1-") && s.ends_with(".tar.gz"))
        })
        .collect::<Vec<_>>();
    files.sort();
    files.reverse();
    for p in files.into_iter().skip(count) {
        fs::remove_file(p).map_err(io)?
    }
    Ok(())
}
pub fn status(c: &ZealotConfig, destination: Option<&Path>) -> Result<Status, String> {
    let dir = destination.unwrap_or_else(|| Path::new(&c.backup_path));
    let latest = fs::read(dir.join("latest-status.json"))
        .ok()
        .and_then(|x| serde_json::from_slice(&x).ok());
    let mut bundles = fs::read_dir(dir)
        .map_err(io)?
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|s| s.starts_with("zealot-backup-v1-") && s.ends_with(".tar.gz"))
        .collect::<Vec<_>>();
    bundles.sort();
    Ok(Status { latest, bundles })
}
pub fn restore(c: &ZealotConfig, bundle: &Path, force: bool) -> Result<(), String> {
    let _lock = Lock::acquire(Path::new(&c.backup_path))?;
    verify(c, bundle)?;
    let media = Path::new(&c.media_path);
    let media_occupied = media.read_dir().ok().and_then(|mut x| x.next()).is_some();
    let occupied = if c.database == "sqlite" {
        Path::new(&c.db_filename).exists() || media_occupied
    } else if c.database == "postgres" {
        media_occupied || postgres_has_tables(c)?
    } else {
        return Err(format!("unsupported database engine {}", c.database));
    };
    if occupied && !force {
        return Err("restore target is non-empty; pass --force after stopping Zealot".into());
    }
    let stage = std::env::temp_dir().join(format!("zealot-restore-{}", Uuid::new_v4()));
    extract_safe(bundle, &stage)?;
    if c.database == "sqlite" {
        let db = Path::new(&c.db_filename);
        if let Some(p) = db.parent() {
            fs::create_dir_all(p).map_err(io)?
        }
        // The complete staged bundle was verified before replacing the target.
        fs::copy(stage.join("database.sqlite"), db).map_err(io)?;
    } else {
        let mut cmd = Command::new("pg_restore");
        cmd.args(pg_client_args(c))
            .args(["--clean", "--if-exists", "--no-owner", "--no-acl"])
            .arg(stage.join("database.dump"));
        if let Some(p) = &c.db_password {
            cmd.env("PGPASSWORD", p);
        }
        let output = cmd
            .output()
            .map_err(|_| "pg_restore is required for PostgreSQL restore".to_string())?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
    }
    if media.exists() {
        fs::remove_dir_all(media).map_err(io)?
    }
    copy_tree(
        &stage.join("media"),
        media,
        &mut BTreeMap::new(),
        &mut 0,
        &mut 0,
    )?;
    verify(c, bundle)?;
    write_receipt(
        Path::new(&c.backup_path),
        &Receipt {
            format_version: FORMAT,
            time: now(),
            reason: "restore".into(),
            outcome: "success".into(),
            bundle: Some(bundle.display().to_string()),
            verification: Some(true),
            error: None,
            pruned: Vec::new(),
        },
    )?;
    let _ = fs::remove_dir_all(stage);
    Ok(())
}

fn pg_client_args(c: &ZealotConfig) -> Vec<String> {
    let mut a = vec![
        "--host".into(),
        c.db_host.clone().unwrap_or_else(|| "localhost".into()),
    ];
    if let Some(u) = &c.db_username {
        a.extend(["--username".into(), u.clone()]);
    }
    if let Some(d) = &c.db_database {
        a.extend(["--dbname".into(), d.clone()]);
    }
    a
}
fn postgres_has_tables(c: &ZealotConfig) -> Result<bool, String> {
    let mut cmd = Command::new("psql");
    cmd.args(pg_client_args(c)).args([
        "--tuples-only",
        "--no-align",
        "--command",
        "SELECT EXISTS (SELECT 1 FROM pg_tables WHERE schemaname='public');",
    ]);
    if let Some(p) = &c.db_password {
        cmd.env("PGPASSWORD", p);
    }
    let out = cmd
        .output()
        .map_err(|_| "psql is required to inspect PostgreSQL restore targets".to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim() == "t")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn archive_paths_are_restricted() {
        assert!(safe(Path::new("media/a")));
        assert!(!safe(Path::new("../x")));
        assert!(!safe(Path::new("/x")));
    }
}
