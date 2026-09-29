# Analysis Tooling

This repository has a local-first analysis layer for Rust and TypeScript. The commands are advisory in the current phase: findings are reported and written to disk, but they do not fail the overall analysis workflow.

## Commands

Run these from the repository root:

```sh
npm run analyze:rust:coupling
npm run analyze:rust:metrics
npm run analyze:ts:deps
npm run analyze:report
npm run analyze:all
```

All generated files are written under `artifacts/analysis/`.

## One-Time Tool Installs

The wrappers are repo-owned, but the analyzers themselves may need to be installed locally.

Rust:

```sh
cargo install cargo-coupling
cargo install rust-code-analysis-cli
```

TypeScript:

```sh
npm install --save-dev dependency-cruiser
```

If a tool is missing, the wrapper prints the exact install command and still exits successfully in advisory mode.

## Current Boundary Model

### Rust

The internal crate/app dependency model is currently:

- `zealot-domain` has no internal dependencies.
- `zealot-app` may depend on `zealot-domain`.
- `zealot-api` and `zealot-infra` may depend on `zealot-app` and `zealot-domain`.
- `zealot-server` may depend on the internal crates.
- `zealot-cli`, `zealot-tui`, and `zealot-mcp` are treated as thin entrypoints and may only depend on `zealot-domain`.

The Rust coupling wrapper enforces those boundaries from `cargo metadata`, then stores a machine-readable summary in `artifacts/analysis/rust-coupling/`.

### TypeScript

The TypeScript dependency rules currently enforce:

- `apps/web` may depend on workspace packages.
- packages under `packages/*` must not import from app code.
- foundational shared packages such as `domain`, `content`, `core`, `commands`, `schemas`, and `theme` must stay below `engine`, `api`, `ui`, and `apps/web`.
- `engine` and `api` must stay below `ui`.
- `ui` must not import from `apps/web`.

The dependency-cruiser config lives at the repo root in `dependency-cruiser.cjs`, and artifacts are written to `artifacts/analysis/ts-deps/`.

## SonarQube Prep

The checked-in `sonar-project.properties` file is local-only preparation. It does not assume a running Sonar service yet.

When scanner activation is added later, provide:

```sh
export SONAR_HOST_URL=...
export SONAR_TOKEN=...
```

Optional future overrides:

```sh
-Dsonar.projectKey=...
-Dsonar.projectName=...
```

The current Sonar config excludes generated, vendor, build, and analysis artifact directories, and keeps scanner metadata under `artifacts/analysis/sonar/`.

## Notes

- Existing legacy docs that describe the old Go/client layout are not the source of truth for this setup.
- `rust-code-analysis` integration is prepared through the wrapper interface today; when the binary is installed, the wrapper captures external tool metadata alongside the local summary.
- `analyze:report` aggregates the per-tool outputs into `artifacts/analysis/summary.md`.
