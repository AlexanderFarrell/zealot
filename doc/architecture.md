# Architecture Overview

Zealot is a monorepo with a browser client, an HTTP API server, and a PostgreSQL data layer.

## High-Level Runtime

1. User accesses the web client.
2. Client calls backend APIs under `/api/...`.
3. Backend enforces session auth, reads/writes PostgreSQL, and returns JSON.
4. Redis is optional/selected for session storage (default in Docker env).
5. Media module stores files in an upload folder on disk.

## Monorepo Layout

- `client/`: SPA-like frontend using custom elements, TypeScript, and Vite.
- `zealotd/`: Go backend using Fiber with modular app packages.
- `database/`: SQL source files merged into `zealotd/init.psql`.
- `cli/`: Cobra-based CLI project (currently scaffolded).
- `dev/`: Docker Compose stack for web, API, DB, and Redis.
- `bin/`: local build output target.

## Core Domains

- Accounts and sessions
- Items and item hierarchy
- Attributes and attribute kinds
- Item types and type assignments
- Planner views by date/week/month/year
- Repeat tracking and completion state
- Comments (day/item scoped timeline entries)
- Media file browsing and upload

## Security and Middleware Notes

Backend middleware stack includes:

- CORS with configurable origins
- Helmet headers
- CSRF token middleware (`X-Csrf-Token`)
- Request limiter
- Request logging

Frontend request helpers automatically acquire and send CSRF tokens for mutating requests.

## Deployment Modes

- Local source workflow via `./zdev`
- Docker Compose stack in `dev/zealot-compose.yml`
- Docker images for:
  - `client` (Nginx serving built static files)
  - `zealotd` (compiled Go binary)
