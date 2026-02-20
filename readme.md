# Zealot

![Zealot Logo](./client/public/zealot.webp)

Zealot is a personal planning and knowledge system with:

- Hierarchical items and notes
- Planner views (daily, weekly, monthly, yearly)
- Repeat tracking
- Item comments and timeline logging
- Item types and custom/system attributes
- Media/file management

The repository is a monorepo containing:

- `client/`: TypeScript + Vite web frontend
- `zealotd/`: Go (Fiber) backend API server
- `database/`: PostgreSQL schema and seed SQL
- `cli/`: Go CLI (currently scaffold-level)
- `dev/`: Docker Compose deployment files

## Documentation

High-level project documentation is in `doc/`:

- `doc/index.md`
- `doc/architecture.md`
- `doc/backend.md`
- `doc/frontend.md`
- `doc/database.md`
- `doc/development.md`

Additional focused docs already in this repository:

- `docs/zealotscript_spec.md`
- `docs/todo.md`

## Quick Start

### Docker (recommended)

1. Configure environment files:
   - `dev/.db.env` (database container settings)
   - `dev/.zealot.env` (backend settings)
   - Example templates are in:
     - `dev/.db.example.env`
     - `dev/.zealot.example.env`
2. Build and start:

```sh
cd dev
docker compose -f zealot-compose.yml up --build
```

Default ports:

- Frontend: `http://localhost:8080`
- Backend API: `http://localhost:8082`
- Postgres: `localhost:5434`
- Redis: `localhost:6379`

### Local development

Use the helper script:

```sh
# Build server, client, cli, and generated DB init script
./zdev build all

# Run backend directly from source
./zdev run server

# Run frontend dev server (Vite)
./zdev run client_dev
```

## Build, Format, and Vet

```sh
# Build components
./zdev build [cli|server|client|database|native|all]

# Formatting
./zdev fmt [cli|server|client|all]

# Go security/static analysis checks
go install honnef.co/go/tools/cmd/staticcheck@latest
./zdev vet [cli|server|all]
```
