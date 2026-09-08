# Troubleshooting and Recovery

This guide covers diagnosing and recovering from common Zealot problems. Start with the section that matches your symptom. If you are not sure what went wrong, start with [Inspecting logs and health](#inspecting-logs-and-health).

---

## Contents

- [Startup failures](#startup-failures)
- [Login and authentication](#login-and-authentication)
- [Migration problems](#migration-problems)
- [Database issues](#database-issues)
- [Browser and client issues](#browser-and-client-issues)
- [Inspecting logs and health](#inspecting-logs-and-health)
- [Backup and restore](#backup-and-restore)
- [Data-loss decision tree](#data-loss-decision-tree)
- [Known limitations and open risks](#known-limitations-and-open-risks)

---

## Startup failures

### The server container exits immediately or stays unhealthy

Check logs first:

```
docker compose logs server
```

The server logs startup in a fixed sequence. Find the last line that printed successfully to know where it failed:

1. Config loaded from environment
2. Database connection established
3. Migrations checked and applied
4. Rules engine initialized
5. Scheduler started
6. HTTP server accepting traffic on port 8456

If the server never reaches step 6, the logs will show why before it exited.

**Common causes:**

| Symptom in logs | Cause | Fix |
|---|---|---|
| `Connection refused` or `could not connect` | Database not reachable | Check `DB_HOST`, `DB_USERNAME`, `DB_PASSWORD`, `DB_DATABASE`. For PostgreSQL, confirm the database and user exist. |
| `checksum mismatch` | A migration file was edited after it was applied | Do not edit migration files. Write a new migration to reverse the change. |
| `error returned from database: …` | SQL error in a migration | See [Migration problems](#migration-problems). |
| `error: … DATABASE …` or missing env var | Required environment variable not set | Check your compose file or env file. |
| Exits silently with no log output | Config parsing failed before logging started | Verify that all required env vars are present. |

### The `web` container stays in `starting`

`web` does not start until `server` passes its health check. If `web` is stuck, the problem is in `server`. Check `docker compose logs server`.

### Port conflict

If port 8085 or 8456 is already in use on the host, Docker will refuse to bind it. The error appears in `docker compose up` output (not container logs):

```
Error response from daemon: driver failed programming external connectivity: Bind for 0.0.0.0:8085 failed: port is already allocated
```

Either stop the conflicting process or change the host-side port in your compose file:

```yaml
ports:
  - "8090:80"    # change 8090 to any free port
```

---

## Login and authentication

### Cannot log in — wrong credentials

There is no self-service password reset. The account and password hash are stored in the `account` table of the database.

If you have database access, you can inspect the table to confirm the account exists:

```
# SQLite
docker run --rm \
  -v zealot_zealot_data:/data \
  alpine sh -c "apk add --no-cache sqlite > /dev/null && sqlite3 /data/zealot.db 'SELECT email FROM account;'"

# PostgreSQL
docker exec <postgres-container> psql -U zealot zealot -c "SELECT email FROM account;"
```

If the account is there and the password is genuinely unknown, the only recovery path is to insert a new password hash or delete the account and re-register. Neither is exposed through the UI — this requires direct database access.

### Session expired or login page keeps reappearing

Sessions are stored in the database. If a session is gone (server restarted, session table purged, cookie cleared), the browser will redirect to login. Log in again — there is no persistent "remember me" mechanism beyond the session cookie lifetime.

### API key returns 401

- The key is shown only once when created. If it was not copied, it cannot be retrieved — generate a new one from account settings.
- Confirm the key is passed as the `X-Api-Key` header, not as a query parameter or bearer token.
- Keys can be revoked from account settings. If the key was revoked, generate a new one.
- Verify the key against a simple endpoint: `curl -H "X-Api-Key: YOUR_KEY" http://your-host:8085/api/item`

### CORS errors in the browser

If the browser console shows a CORS error (`Access to fetch at '…' from origin '…' has been blocked`), the cause is a known limitation: **CORS is hardcoded to `tauri://localhost`** in the server source (`crates/zealot-api/src/http/mod.rs`, line 36).

Any browser-based access from a real domain will fail CORS. The fix requires a source change:

1. Edit `crates/zealot-api/src/http/mod.rs` — change the `allow_origin` value to your deployment domain (e.g. `https://zealot.example.com`).
2. Rebuild the server image.
3. Redeploy.

There is no environment variable for this.

### CSRF errors from API scripts

External scripts that authenticate via session cookie also need to send an `X-Csrf-Token` header, which the browser frontend handles automatically but scripts do not. The simpler fix: use an API key (`X-Api-Key`) instead of a session cookie for programmatic access. API key clients are exempt from CSRF requirements.

---

## Migration problems

### Server refuses to start: "checksum mismatch"

sqlx checksums every migration file when it starts. If a file that was already applied has been modified, the server exits with an error like:

```
migration 3/0003_backfill_parent_links.sql has a different checksum than the one applied to the database
```

**Do not attempt to fix this by editing `_sqlx_migrations` directly.**

The correct fix: write a new migration that achieves the desired schema state. The original file must be restored to exactly what was applied, or left as-is. If you do not have the original file, check git history.

### Server refuses to start: SQL error in a migration

The log will show the migration version number and the SQL error. For example:

```
error: error returned from database: column "foo" already exists, migration: 4/0004_add_foo_column
```

This means the migration SQL is invalid for the current database state. Common causes: the column already exists (migration was partially applied manually), a referenced table does not exist, or a SQLite-incompatible statement in the SQLite migration.

Steps:
1. Read the failing migration file in `crates/zealot-infra/migrations/<engine>/`.
2. Inspect the database to understand the current schema.
3. If the schema already reflects the migration's intent, mark the migration as applied manually by inserting a row into `_sqlx_migrations` — but do this only if you are certain the schema is correct.
4. If the schema is wrong, fix it manually, then re-run.

Safer approach: restore from the last good backup and investigate on a copy.

### Confirming which migrations have been applied

```
# SQLite
docker run --rm \
  -v zealot_zealot_data:/data \
  alpine sh -c "apk add --no-cache sqlite > /dev/null && sqlite3 /data/zealot.db 'SELECT version, description, success, installed_on FROM _sqlx_migrations ORDER BY version;'"

# PostgreSQL
docker exec <postgres-container> psql -U zealot zealot \
  -c "SELECT version, description, success, installed_on FROM _sqlx_migrations ORDER BY version;"
```

A `success = false` row (or a missing row for a version the image includes) tells you where the sequence broke.

### Rolling back a migration

There is no built-in rollback command. To revert to a pre-migration state:

1. Stop the server: `docker compose stop server`
2. Restore the database from a backup taken before the migration (see [Backup and restore](#backup-and-restore)).
3. Change the image tag in your compose file to the previous version.
4. Start: `docker compose start server`

The server will re-apply any migrations that were in the backup-point image but not yet in the backup. This is expected and safe.

---

## Database issues

### SQLite: database file not found or permission error

The database file lives at `/data/zealot.db` inside the `server` container, on the `zealot_data` volume. Confirm the volume is mounted:

```
docker volume inspect zealot_zealot_data
```

The `Mountpoint` field shows the host path. If the volume does not exist, it was either never created or was deleted with `docker compose down -v`.

If the file exists but the server cannot read it, check file ownership in the volume:

```
docker run --rm -v zealot_zealot_data:/data alpine ls -la /data/
```

The file should be owned by root (UID 0) for the default image.

### SQLite: disk full

The server will log write errors when disk space is exhausted. The database may be in an inconsistent state. Steps:

1. Free disk space on the host.
2. Do not restart the server until you have inspected the database:
   ```
   docker run --rm -v zealot_zealot_data:/data alpine sh -c \
     "apk add --no-cache sqlite > /dev/null && sqlite3 /data/zealot.db 'PRAGMA integrity_check;'"
   ```
3. If `integrity_check` returns `ok`, restart the server normally.
4. If it returns errors, restore from backup.

### PostgreSQL: connection refused

Confirm the PostgreSQL container or server is running and reachable from the `server` container. Check that `DB_HOST` matches the Docker Compose service name (e.g. `postgres`) or the actual hostname, not `localhost` (which resolves to the `server` container itself).

### PostgreSQL: authentication failed

The `DB_USERNAME` and `DB_PASSWORD` values must match what was created in PostgreSQL. Confirm:

```
docker exec <postgres-container> psql -U zealot zealot -c "SELECT 1;"
```

If this fails, the credentials are wrong or the user does not exist.

### PostgreSQL: database does not exist

Zealot does not create the database. Before the first start:

```sql
CREATE USER zealot WITH PASSWORD 'your-password';
CREATE DATABASE zealot OWNER zealot;
```

---

## Browser and client issues

### Blank page or the app does not load

1. Open browser developer tools → Console. Look for errors.
2. Check that the `web` container is running and healthy: `docker compose ps`
3. Confirm the URL is correct: `http://your-host:8085/`
4. If the console shows a network error or 502, check `docker compose logs web` — nginx may have failed to reach the server upstream.

### nginx 502 Bad Gateway

nginx proxies `/api/` to the server container. If the server is down or not yet healthy, nginx returns 502. Check `docker compose logs server` to diagnose the server problem.

### nginx upstream error after changing deployment topology

The web container proxies `/api/` to `API_UPSTREAM`. Set it to an address reachable
from the web container, including its scheme, then recreate the web service. For
example, the bundled Compose and Swarm deployments use `http://server:8456`; a
separate API host could use `https://api.zealot.example`.

### All API requests return CORS errors

See [CORS errors in the browser](#cors-errors-in-the-browser). This is a known source-level limitation.

---

## Inspecting logs and health

### Viewing logs

```
docker compose logs -f server    # follow server logs
docker compose logs -f web       # follow nginx logs
docker compose logs server       # dump all server logs without following
```

Add `--since 10m` to limit to recent output:

```
docker compose logs --since 10m server
```

### Increasing log verbosity

Set the `RUST_LOG` environment variable on the `server` container:

| Value | Effect |
|---|---|
| `info` | One line per request, startup events. Normal operation. |
| `zealot=debug` | Verbose output from Zealot's own code. Use this when investigating application logic. |
| `debug` | Very noisy — includes sqlx query logs and tower internals. Use briefly. |

In your compose file:

```yaml
environment:
  RUST_LOG: zealot=debug
```

Restart the server after changing it: `docker compose up -d server`

### Health endpoints

Both endpoints return a plain-text body with HTTP 200 when the server is healthy.

```
GET /health        → "ok"     (server process is alive)
GET /health/ready  → "ready"  (server has finished startup, including migrations)
```

Call them via the web frontend port (which proxies `/api/` — note: health is at root path on the server, accessible via the internal port):

```
# Via the internal server port (if exposed)
curl http://your-host:8456/health
curl http://your-host:8456/health/ready

# Via the web frontend (nginx proxies /api/ but not /health directly)
# Use the server port for health checks
```

From outside the host, if port 8456 is not exposed (correct for production), check health from inside the Docker network:

```
docker exec $(docker compose ps -q server) wget -qO- http://localhost:8456/health
```

---

## Backup and restore

Create a verified bundle without stopping a running server:

```
docker compose exec server zealot-server backup create --json
docker compose exec server zealot-server backup status --json
docker compose exec server zealot-server backup verify /backups/zealot-backup-v1-....tar.gz --json
```

To recover, stop the server and run the operator command with the same `/backups` mount:

```
docker compose stop server
docker compose run --rm server zealot-server restore /backups/zealot-backup-v1-....tar.gz --force --json
docker compose up -d server
```

`BACKUP_ENABLED=true` enables the UTC `BACKUP_SCHEDULE` (default `0 2 * * *`); the default retention is seven verified bundles. Inspect `/backups/attempts.jsonl` and `/backups/latest-status.json` after a failed scheduled job or restore. A backup mount on the same host is not host-loss protection: replicate verified bundles elsewhere.

---

## Data-loss decision tree

Work through these steps in order. Do not delete or overwrite anything until you have confirmed what is and is not present.

**Step 1 — Is the server running?**

```
docker compose ps
curl http://localhost:8456/health    # or via docker exec if port is not exposed
```

If the server is not running, fix the startup problem first (see [Startup failures](#startup-failures)). A server that is not running is not the same as lost data.

**Step 2 — Can you log in?**

Try logging in via the web UI. If login fails due to a session or cookie issue, clear browser cookies and try again. If credentials are rejected, see [Login and authentication](#login-and-authentication).

**Step 3 — Is the data actually missing?**

Query the API directly to rule out a UI rendering problem:

```
curl -H "X-Api-Key: YOUR_KEY" http://your-host:8085/api/item
```

If the API returns your items, the problem is in the frontend, not the data.

**Step 4 — Is the Docker volume intact?**

```
docker volume inspect zealot_zealot_data
```

If the volume exists and has a `Mountpoint`, the data directory is present. Inspect its contents:

```
docker run --rm -v zealot_zealot_data:/data alpine ls -lh /data/
```

Confirm `zealot.db` (SQLite) or that the database service is reachable (PostgreSQL).

**Step 5 — Was `docker compose down -v` run?**

`docker compose down -v` removes volumes. If this was run, the data is gone unless a backup exists. Check backup files and restore (see [Backup and restore](#backup-and-restore)).

If you are not sure whether `-v` was used: `docker volume ls | grep zealot`. If the volume is absent, it was deleted.

**Step 6 — Was an upgrade done recently?**

Upgrades run new migrations but do not alter existing data. Check the migration log at startup:

```
docker compose logs server | grep -i migrat
```

If migrations ran cleanly, data loss from an upgrade is unlikely. The data should be there — check the API directly (Step 3).

**Step 7 — What not to do**

- Do not run `docker compose down -v` at any point during investigation. It will delete the volume.
- Do not overwrite or move the database file before you have confirmed it is corrupt. The file on disk is the primary copy.
- Do not restart the server repeatedly. Repeated restarts do not recover data and may obscure the actual error in log output.
- Do not truncate or drop tables in an attempt to fix a migration error without first taking a backup.

**Step 8 — When to restore from backup**

Restore from backup when:

- The volume is confirmed deleted (`docker volume ls` shows no `zealot_zealot_data`).
- `sqlite3 /data/zealot.db 'PRAGMA integrity_check;'` returns errors.
- A migration put the schema into a broken state and you cannot recover it forward.
- Data is confirmed missing from the API response and the database file/service is healthy (i.e. data was deleted through the application or directly).

Before restoring: note the current state. If the database file is present and possibly partially valid, copy it somewhere before overwriting.

---

## Known limitations and open risks

The following are current gaps that affect production deployments and recovery scenarios. They are documented here so you can plan for them.

1. **CORS is hardcoded to `tauri://localhost`.** Browser-based access from any real domain fails. Requires a source change in `crates/zealot-api/src/http/mod.rs` and an image rebuild. No env var workaround exists.

2. **Port 8456 is published to the host by default** in both `docker-compose.yml` and `scripts/zealot-compose.yml`. Remove this mapping in production to prevent direct API access bypassing nginx.

3. **The nginx upstream must be reachable from the web container.** Configure
   `API_UPSTREAM` with a full URL such as `http://server:8456`; changing it
   requires recreating the web service, but not rebuilding the image.

4. **No built-in backup scheduling.** Zealot does not run its own backup jobs. This is the operator's responsibility. Data loss from hardware failure or accidental volume deletion is unrecoverable without a backup.

5. **`ACCOUNT_CREATION_ENABLED` is not wired into the server binary.** It appears in example env files but has no effect. Blocking new account registration requires a deny rule at the reverse proxy level.

6. **No migration rollback mechanism.** Once a migration has been applied, reverting requires restoring a database backup and redeploying an older image. There is no `migrate down` command.

7. **No self-service password reset.** Forgotten passwords require direct database access to update the password hash or delete and re-create the account.
