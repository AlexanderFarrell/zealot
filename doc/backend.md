# Backend (zealotd)

## Stack

- Go
- Fiber v2
- PostgreSQL (`lib/pq`)
- Optional Redis-backed session storage

## Entry Point

- `zealotd/server.go`

Startup flow:

1. Connect database.
2. Initialize schema if DB is empty (from embedded `init.psql`).
3. Initialize session store.
4. Register app routers.
5. Start Fiber server.

## Router Modules

Routers currently initialized in `zealotd/server.go`:

- `/account`
- `/item` (includes nested item-type and attribute-kind routes)
- `/planner`
- `/media`
- `/repeat`
- `/comments`

Other app modules exist in tree (for example tracker/analysis/rules), but not all are wired from the current server entrypoint.

## Major API Areas

`/account`:

- register, login, logout
- session status
- account details
- account settings patch

`/item`:

- root item listing
- get by title/id
- search
- children/related
- create, update, delete
- set/rename/delete attributes
- assign/unassign item types

`/planner`:

- items for day/week/month/year

`/repeat`:

- get repeat entries for a day
- set repeat status/comment for item/day

`/comments`:

- get comments by day
- get comments by item
- create/update/delete comment entries

`/media`:

- list/read path
- upload/post
- mkdir
- rename
- delete path

## Authentication and Sessions

- Most domain routes use `RequireLoginMiddleware`.
- Session keys include account identity (`account_id`, `username`).
- Session store:
  - in-memory default
  - Redis when `SESSION_STORE=redis`

## Configuration (selected env vars)

Server/network:

- `PORT`
- `CORS_ALLOW_ORIGINS`

Database:

- `DB_HOSTNAME`
- `DB_PORT`
- `DB_DATABASE`
- `DB_USERNAME`
- `DB_PASSWORD`
- `DB_SSL_MODE` and optional SSL cert/key vars

Session/cookies:

- `SESSION_STORE`
- `SESSION_COOKIE_HTTPONLY`
- `SESSION_COOKIE_SECURE`
- `SESSION_COOKIE_SAMESITE`
- `REDIS_HOST`, `REDIS_PORT`, `REDIS_DATABASE`, `REDIS_PASSWORD`

Account/media:

- `ACCOUNT_CREATION_ENABLED`
- `ACCOUNT_SALT_ROUNDS`
- `UPLOAD_FOLDER`
