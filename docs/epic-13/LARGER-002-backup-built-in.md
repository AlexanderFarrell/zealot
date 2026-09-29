# LARGER-002 — Built-in scheduled backup mechanism (reference: ops / infrastructure epic)

## Why this came from documentation

Both `deployment.md` and `troubleshooting.md` list this as a known limitation:

> "No built-in backup scheduling. Zealot does not run its own backup jobs. This is
> the operator's responsibility."

The troubleshooting doc's data-loss decision tree notes:

> "Data loss from hardware failure or accidental volume deletion is unrecoverable
> without a backup."

The docs provide a `crontab` example for SQLite backups, but this is an external
responsibility. For a personal self-hosted tool, "configure a host-level cron job"
is a meaningful friction point — many operators won't do it, and will lose data.

## Pain points identified

1. The backup steps require running `docker stop`, then a separate `docker run` to
   copy the file, then `docker start`. This is error-prone for operators unfamiliar
   with Docker volumes.
2. No retention policy guidance beyond "7-day retention is reasonable."
3. PostgreSQL backup requires knowing the Postgres container name, which varies.
4. There is no way to know if backups are actually succeeding.

## Proposed direction (for a future ops epic)

**Option A — Backup sidecar container:**
A lightweight sidecar container (`zealot-backup`) mounts the same volume as `server`
and runs periodic `sqlite3 .backup` (hot backup, no need to stop server) or `pg_dump`
on a cron schedule. Results go to a configurable target (local path, S3, etc.).

**Option B — Built-in backup endpoint:**
Add `POST /admin/backup` to the server that triggers an in-process SQLite checkpoint
and streams the `.db` file as a download. Operators can call this from a cron job
without Docker volume gymnastics. Simpler but doesn't handle PostgreSQL.

**Option C — Documentation-only improvement:**
Add a `scripts/backup.sh` helper that wraps the Docker volume copy commands, and
document how to register it as a systemd timer.

## Belongs in

A dedicated **operations / infrastructure epic**, covering:
- Automated backup configuration
- Backup retention and rotation
- Restore validation tooling
- Optionally: S3/object storage media backend (currently only `FILESYSTEM` is
  supported, which complicates remote backups)
