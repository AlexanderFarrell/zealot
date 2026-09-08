# Deployment and Operations

This guide covers running Zealot persistently on a server or dedicated machine — configuration, data, upgrades, backups, and security. If you are running Zealot for the first time on a local machine, start with [Quickstart](./quickstart.md). If you need to build images from source, see [Building & Running](./building.md).

Docker Compose is the supported deployment method. All examples assume you have Docker and Compose installed and working.

---

## Services and ports

Zealot is composed of three containers.

| Service | Image | Port | Role | Health check |
|---|---|---|---|---|
| `server` | `zealotd` | 8456 | Rust API backend — all data reads and writes | `GET /health` |
| `web` | `zealot-web` | 8085 (→ 80 internally) | nginx — serves the SPA, proxies `/api/` to the server | `GET /` |
| `mcp` | `zealot-mcp` | 3100 | MCP integration for AI agents (optional) | TCP port 3100 |

`web` does not start until `server` passes its health check (20 retries over ~100 seconds). `mcp` also depends on `server`.

**Port 8456 is internal.** In production, the server port should not be reachable from outside the host. Only port 8085 (the nginx frontend) should be publicly accessible. The [reverse proxy section](#reverse-proxy-and-tls) covers how to enforce this.

---

## Data storage

All persistent state lives in a single Docker named volume: `zealot_data`. Inside the `server` container it is mounted at `/data`.

```
/data/
├── zealot.db      ← SQLite database (when using DATABASE=sqlite)
└── public/        ← uploaded media files
```

The `web` and `mcp` containers are stateless — only `server` needs the volume.

The volume persists across `docker compose down` and image updates. **`docker compose down -v` removes the volume and all data in it.** Do not use `-v` unless you intend to wipe everything.

When referencing the volume via the Docker CLI, it is prefixed with the Compose project name. If your project is named `zealot` (the default), the volume is `zealot_zealot_data`:

```
docker volume inspect zealot_zealot_data
```

The `Mountpoint` field shows the host path (typically `/var/lib/docker/volumes/zealot_zealot_data/_data`).

---

## Environment variables

### Server (`server` container)

| Variable | Default in Docker | Required | Notes |
|---|---|---|---|
| `DATABASE` | `sqlite` | Yes | `sqlite` or `postgres` |
| `DB_FILENAME` | `/data/zealot.db` | If SQLite | Full path to the database file |
| `DB_HOST` | — | If PostgreSQL | Hostname of the PostgreSQL server |
| `DB_USERNAME` | — | If PostgreSQL | PostgreSQL user |
| `DB_PASSWORD` | — | If PostgreSQL | **Secret.** Do not commit to version control. |
| `DB_DATABASE` | — | If PostgreSQL | PostgreSQL database name |
| `MEDIA_SOURCE` | `FILESYSTEM` | No | Only `FILESYSTEM` is currently supported |
| `MEDIA_PATH` | `/data/public` | No | Path where uploaded files are written |
| `PORT` | `8456` | No | Internal HTTP listen port |

The defaults shown are those baked into the server Dockerfile (`apps/server/Dockerfile`), not the `config.rs` code defaults. The compose files override `DATABASE=sqlite` explicitly — this is intentional.

### MCP (`mcp` container)

| Variable | Default | Required | Notes |
|---|---|---|---|
| `ZEALOT_URL` | `http://localhost:7377` | Yes | URL of the Zealot API server. Use the Docker service name: `http://server:8456` |
| `ZEALOT_API_KEY` | — | **Yes** | API key generated in Zealot account settings. **Secret.** |
| `MCP_MODE` | `stdio` | No | `stdio` (Claude Desktop) or `http` (container deployment) |
| `MCP_PORT` | `3100` | No | Port for HTTP mode |

`zealot-mcp` will refuse to start if `ZEALOT_API_KEY` is empty.

---

## Database

### Choosing SQLite or PostgreSQL

**SQLite** is the right choice for personal use, a single user, or any instance that does not need to be queried by external tooling. It requires no separate database service. The database is a single file on the volume that you can copy for backups.

**PostgreSQL** is appropriate if you want to run standard database tooling against Zealot's data, need higher concurrent write throughput, or prefer the backup semantics of `pg_dump`. It requires a running PostgreSQL server that Zealot can reach.

### PostgreSQL prerequisites

Zealot creates its own tables but does not create the database or user. Before starting Zealot with `DATABASE=postgres`, create both:

```sql
CREATE USER zealot WITH PASSWORD 'your-password';
CREATE DATABASE zealot OWNER zealot;
```

Then set the corresponding env vars: `DB_HOST`, `DB_USERNAME`, `DB_PASSWORD`, `DB_DATABASE`.

### Migrations

Zealot runs all pending schema migrations automatically at startup, before the server accepts any traffic. There is no separate migration command to run. The sequence is:

1. Connect to the database
2. Check `_sqlx_migrations` for already-applied versions
3. Run any new migration files in version order, each in a transaction
4. Start accepting traffic

If any migration fails, the server exits with an error in the logs. It will not start in a partially-migrated state.

**Do not edit migration files after they have been applied.** sqlx checksums every file; a modification to an already-run migration causes the server to refuse to start. To reverse a change, write a new migration.

See [Migrations](./migrations.md) for the full migration system reference.

### Upgrading

Pull the new image and restart. That is all.

```
docker compose pull
docker compose up -d
```

The server applies any new migrations on startup. If the server container exits during startup, check the logs — a failed migration produces a specific error before exit.

### Rollback

There is no built-in rollback mechanism. If a new release is broken and you need to revert:

1. Stop the containers.
2. Restore the database from a backup taken before the upgrade (see [Backup and restore](#backup-and-restore)).
3. Redeploy the previous image tag.

This is why taking a backup before each upgrade is strongly recommended.

---

## Running with Docker Compose

### Pre-built images (recommended)

`scripts/zealot-compose.yml` uses published images and is the starting point for server deployments:

```
docker compose -f scripts/zealot-compose.yml up -d
```

The web frontend is served at **http://your-host:8085**.

To watch startup logs:

```
docker compose -f scripts/zealot-compose.yml logs -f
```

### Building from source

The root `docker-compose.yml` builds both images locally. Use this if you are running a fork or a branch that is not published:

```
docker compose up -d --build
```

The first build takes several minutes (Rust compilation). Subsequent builds are faster due to Docker layer caching.

### Checking status

```
docker compose ps
```

All containers should show as `healthy`. If any show as `starting` for more than a minute or `unhealthy`, check logs:

```
docker compose logs server
docker compose logs web
```

### Updating to a new version

```
docker compose pull
docker compose up -d
```

Running containers are replaced one at a time. Data is untouched.

### Stopping and starting

```
docker compose stop       # stops containers, volume intact
docker compose start      # resumes them
```

Or to tear down and recreate:

```
docker compose down       # removes containers, volume intact
docker compose up -d      # starts fresh, migrations run again
```

Again: **do not use `docker compose down -v`** unless you want to delete all data.

### Docker Swarm

For Swarm deployments, use `docker stack deploy`:

```
docker stack deploy -c scripts/zealot-compose.yml zealot
```

To redeploy after an image update:

```
docker stack rm zealot
docker stack deploy -c scripts/zealot-compose.yml zealot
```

---

## Reverse proxy and TLS

For any deployment reachable from outside your home network, you need a reverse proxy that terminates TLS. Do not expose Zealot over plain HTTP to the internet — credentials and session cookies travel in the clear.

The intended architecture is:

```
Internet ──HTTPS──► reverse proxy (443) ──HTTP──► zealot-web (8085)
```

### CORS limitation — read before proceeding

**CORS is currently hardcoded to `tauri://localhost` in the server source** (`crates/zealot-api/src/http/mod.rs`, line 36). This means browser-based API requests from any real domain will be blocked by the browser's CORS check. Every request will fail with a CORS error.

Before deploying Zealot behind a reverse proxy and accessing it from a browser at a real URL, you must:

1. Change the `allow_origin` value in `crates/zealot-api/src/http/mod.rs` to your deployment domain (e.g. `https://zealot.example.com`).
2. Rebuild the server image.

There is no environment variable for this — it requires a source change. This is a known limitation.

### Port binding for production

In the compose file, remove the server port mapping entirely and bind the web port to localhost so only the reverse proxy can reach it:

```yaml
services:
  server:
    # no ports: entry — server is internal only
    ...

  web:
    ports:
      - "127.0.0.1:8085:80"    # only the reverse proxy can reach this
    ...
```

### nginx reverse proxy

```nginx
server {
    listen 443 ssl;
    server_name zealot.example.com;

    ssl_certificate     /etc/ssl/certs/zealot.example.com.crt;
    ssl_certificate_key /etc/ssl/private/zealot.example.com.key;

    location / {
        proxy_pass         http://127.0.0.1:8085;
        proxy_http_version 1.1;
        proxy_set_header   Host              $host;
        proxy_set_header   X-Real-IP         $remote_addr;
        proxy_set_header   X-Forwarded-For   $proxy_add_x_forwarded_for;
        proxy_set_header   X-Forwarded-Proto $scheme;
    }
}

server {
    listen 80;
    server_name zealot.example.com;
    return 301 https://$host$request_uri;
}
```

### Caddy

```
zealot.example.com {
    reverse_proxy localhost:8085
}
```

Caddy handles certificate provisioning automatically via Let's Encrypt.

---

## Security

### Network exposure

Remove the `8456:8456` port mapping from your production compose file. The server API is an internal service — external traffic should only reach it through the nginx frontend. If port 8456 is reachable from the internet, it bypasses the nginx layer entirely.

### Authentication model

- **Browser clients** authenticate with session cookies. Login via the UI sets a `Set-Cookie` header; the browser sends it automatically on subsequent requests.
- **Programmatic clients** (mobile app, MCP, scripts) use API keys. Generate a key in account settings. Pass it as the `X-Api-Key` header. Keys are shown only once on creation — copy immediately. Revoke unused keys from account settings.
- **Mutation protection**: state-changing requests require an `X-Csrf-Token` header. The frontend handles this automatically. External API clients using `X-Api-Key` are exempt from CSRF requirements.

### TLS

Do not deploy without TLS for any instance accessible over the internet or a network you do not control. Session cookies and API keys are credentials — they must not travel in the clear.

### CORS

As described in the [reverse proxy section](#cors-limitation--read-before-proceeding), CORS is hardcoded. No source change = no working web access from a real domain.

### Database credentials

If using PostgreSQL, do not put `DB_PASSWORD` in a compose file that is committed to version control or shared. Use a separate env file:

```
# .zealot.env — exclude from version control
DB_PASSWORD=your-password
ZEALOT_API_KEY=your-mcp-key
```

Reference it from your compose file:

```yaml
services:
  server:
    env_file: .zealot.env
```

Add `.zealot.env` to `.gitignore`.

### Locking account registration

The example env files reference `ACCOUNT_CREATION_ENABLED`, but this variable is not wired into the server binary at the time of writing (`crates/zealot-app/src/config.rs` does not read it). Do not rely on it to prevent new registrations. If you need to block the register endpoint, do it at the reverse proxy level with a `deny` rule on the registration route.

### Least privilege

The server container runs a single binary. It does not need host networking, privileged mode, or extra Linux capabilities. The default Docker security model is sufficient.

If you mount a host directory for media storage instead of using the named volume, ensure the directory is writable by the container's process (UID 0 in the default image).

---

## Backup and restore

Backups are operator commands, not HTTP endpoints. Bundles contain a database snapshot, filesystem media, a versioned manifest, and SHA-256 evidence. They are unencrypted: the backup mount must have restrictive permissions.

The Compose files mount a separate `zealot_backups` volume at `/backups`. It survives container replacement, but storage on the same host does **not** protect against host loss. Replicate verified bundles to separate storage.

### Create, verify, and inspect

```
# Runs an online SQLite snapshot (or pg_dump custom format), then verifies it.
docker compose exec server zealot-server backup create --json
docker compose exec server zealot-server backup status --json
docker compose exec server zealot-server backup verify /backups/zealot-backup-v1-....tar.gz --json
```

Use `--destination PATH` for a one-off location. Successful verified bundles are retained newest-first according to `BACKUP_RETENTION_COUNT` (default `7`). `attempts.jsonl` and `latest-status.json` in the backup directory preserve both success and failure evidence.

### Scheduling and upgrades

Set `BACKUP_ENABLED=true` to enable the UTC cron schedule. Defaults are `BACKUP_PATH=/backups`, `BACKUP_SCHEDULE="0 2 * * *"`, and `BACKUP_RETENTION_COUNT=7`. A scheduled failure is recorded and logged without stopping an already-running server. On non-fresh startup, an enabled backup is created and verified before migrations; failure prevents migration.

Before an upgrade, run `zealot-server backup create --json`, verify the returned bundle, then deploy. For Swarm, mount a distinct persistent volume or host path at `/backups`; for a direct local run set `BACKUP_PATH` to a protected persistent directory.

### Restore

Stop Zealot before restoring. Restore refuses a populated target unless `--force` is present and validates the archive, manifest, hashes, integrity, and counts before replacing data:

```
docker compose stop server
docker compose run --rm server zealot-server restore /backups/zealot-backup-v1-....tar.gz --force --json
docker compose up -d server
```

For a clean-environment recovery, start the same image with empty `/data` and the existing `/backups` mount, run the restore command above, then start the server. The successful restore appends a versioned receipt to `attempts.jsonl`. A bundle can only be restored into the matching configured database engine; SQLite-to-PostgreSQL conversion and remote/S3 storage are intentionally out of scope.

---

## Logging and observability

All server logs go to stdout and are captured by Docker.

```
docker compose logs -f server
docker compose logs -f web
```

### Log verbosity

Set the `RUST_LOG` environment variable on the `server` container:

| Value | Effect |
|---|---|
| `info` | One line per request, startup events. Normal operation. |
| `zealot=debug` | Verbose output from Zealot's own code. |
| `debug` | Very noisy — includes sqlx query logs, tower internals. Use briefly for debugging. |

Add to your compose file:

```yaml
environment:
  RUST_LOG: info
```

### Startup sequence

A healthy server startup produces log lines in this order:

1. Config loaded from environment
2. Database connection established
3. Migrations checked and applied
4. Rules engine initialized
5. Scheduler started
6. HTTP server accepting traffic on port 8456

If the server exits before step 6, the last log lines identify the cause. Migration failures are the most common reason — sqlx logs the specific SQL error and version number.

---

## Verification checklist

Run through this list after any new deployment or after an upgrade.

```
[ ] docker compose ps — all containers show as "healthy"
[ ] http://your-host:8085/ — Zealot login screen loads
[ ] http://your-host:8085/api/health — returns HTTP 200
[ ] Login with your account credentials succeeds
[ ] Create a new item, set an attribute, verify it saves and reloads
[ ] Port 8456 is NOT reachable externally:
      curl --max-time 3 http://your-external-ip:8456/health
      (should time out or be refused — not return 200)
[ ] If TLS: https://your-domain/ loads without certificate warnings
[ ] If TLS: http://your-domain/ redirects to https://
[ ] docker compose down && docker compose up -d — data is still present after restart
[ ] docker compose logs server — no errors in the migration section at startup
[ ] API key authentication:
      curl -H "X-Api-Key: YOUR_KEY" http://your-host:8085/api/item
      (should return JSON, not 401)
[ ] If MCP deployed: http://your-mcp-host:3100/ — returns 200 OK
```

---

## Known limitations

The following are current gaps that affect production deployments. They are not gotchas — they are documented here so you know what to plan for.

1. **CORS is hardcoded to `tauri://localhost`.** Web deployments from a real domain require a source change in `crates/zealot-api/src/http/mod.rs` and an image rebuild. There is no env var workaround.

2. **Port 8456 is published to the host by default** in both `docker-compose.yml` and `scripts/zealot-compose.yml`. Remove this mapping in production.

3. **The nginx upstream is hardcoded.** `apps/web/nginx.conf` references `zealot_zealot:8456` as the upstream. This is the Docker Compose internal DNS name for a project named `zealot`. If you change the project name or service name, the web image must be rebuilt with an updated `nginx.conf`.

4. **No built-in backup scheduling.** Zealot does not run its own backup jobs. This is the operator's responsibility.

5. **`ACCOUNT_CREATION_ENABLED` is not wired into the server binary.** It appears in example env files but does not affect the running server. Blocking registration requires a reverse proxy rule.
