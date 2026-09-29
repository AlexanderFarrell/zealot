# Development Guide

## Prerequisites

- Go (compatible with backend/CLI module versions)
- Node.js + npm
- Docker + Docker Compose (for container workflow)
- PostgreSQL and Redis (if running dependencies locally outside Docker)

## Local Commands (`zdev`)

The repository root includes `zdev`, a Python helper script.

Build:

```sh
./zdev build [cli|server|client|docker|database|native|all]
```

Run:

```sh
./zdev run server
./zdev run client_dev
./zdev run cli
```

Formatting:

```sh
./zdev fmt [server|client|cli|all]
```

Vetting:

```sh
go install honnef.co/go/tools/cmd/staticcheck@latest
./zdev vet [server|cli|all]
```

## Docker Workflow

Compose file: `dev/zealot-compose.yml`

Services:

- `db` (Postgres)
- `redis`
- `zealot` (Go backend)
- `web_ui` (Nginx static frontend)

Run:

```sh
cd dev
docker compose -f zealot-compose.yml up --build
```

Default ports:

- `8080`: frontend
- `8082`: backend
- `5434`: postgres host mapping
- `6379`: redis

## Environment Files

- `dev/.db.env`: Postgres container variables
- `dev/.zealot.env`: backend app variables
- Templates:
  - `dev/.db.example.env`
  - `dev/.zealot.example.env`

## Frontend Workflow

```sh
cd client
npm install
npm run dev
```

Dev server proxies `/api` to backend (`client/vite.config.ts`).

## Backend Workflow

```sh
cd zealotd
go run .
```

Key startup behavior:

- connects DB
- initializes DB if empty
- starts sessions
- starts HTTP server

## CLI Workflow

The CLI project (`cli/`) currently exists as a Cobra scaffold and can be built/run, but command behaviors are minimal placeholders.
