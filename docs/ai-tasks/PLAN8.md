# Mixed-Language Analysis Tooling Plan

## Summary
Add a repo-owned analysis layer for Rust and TypeScript that is local-first, advisory-first, and built around architectural boundary checks rather than immediate hard CI gates.

The implementation should expose a small set of root commands that orchestrate:
- Rust structure and code metrics via `cargo-coupling` and `rust-code-analysis`
- TypeScript dependency graph and rule enforcement via `dependency-cruiser`
- Aggregated quality reporting prepared for later SonarQube adoption, but without assuming a live Sonar service yet

## Key Changes
### Root analysis entrypoints
Add root-level npm scripts as the primary interface, with names under an `analyze:*` namespace:
- `analyze:rust:coupling`
- `analyze:rust:metrics`
- `analyze:ts:deps`
- `analyze:all`
- `analyze:report`

Use repo wrapper scripts under `scripts/analysis/` to normalize invocation, output paths, and exit behavior. These wrappers should:
- check tool presence and print exact install commands when missing
- write artifacts into a single ignored directory such as `artifacts/analysis/`
- return success by default in advisory mode, even when findings exist
- fail only on invocation/config errors, not on detected code smells or dependency violations yet

### Rust analysis integration
Scope Rust analysis to the Cargo workspace discovered in the repo:
- apps: `server`, `cli`, `tui`, `mcp`
- crates: `zealot-domain`, `zealot-app`, `zealot-infra`, `zealot-api`

Implement `cargo-coupling` checks against the current layered dependency model:
- `zealot-domain` is the base domain crate
- `zealot-app` may depend on `zealot-domain`
- `zealot-api` and `zealot-infra` may depend on `zealot-app` and `zealot-domain`
- `apps/server` may depend on the internal crates
- `apps/cli`, `apps/tui`, and `apps/mcp` should remain thin entrypoints and not introduce sideways dependencies on infra/api unless explicitly allowed by the wrapper config

Use `rust-code-analysis` to generate non-blocking metrics for Rust sources:
- complexity / maintainability style metrics
- function/file hotspots
- duplicate-code signal if supported by the selected invocation mode

Persist Rust outputs as machine-readable files where possible so later SonarQube ingestion or dashboarding does not require rework.

### TypeScript dependency analysis
Add a root `dependency-cruiser` config for the npm workspace:
- cover `apps/web` and the `packages/*` workspace packages
- ignore generated/vendor paths like `node_modules`, build output, and test reports
- model current internal package relationships as allowed boundaries

Enforce layer boundaries in advisory mode:
- `apps/web` may depend on published workspace packages
- shared packages should not import from app code
- lower-level shared packages such as domain/content/core should not depend on UI-facing packages
- `packages/ui` may depend on lower-level packages, but lower-level packages must not depend on `packages/ui`

Emit at least:
- a text summary for terminal use
- a graph artifact such as DOT/SVG/HTML if practical
- a machine-readable JSON report for later aggregation

### SonarQube preparation
Add repo-side SonarQube prep only, without assuming a running server yet:
- check in a root `sonar-project.properties` or equivalent documented config
- define source/test exclusions for Rust, TS, generated files, reports, and `node_modules`
- point Sonar inputs at the analysis artifact directory so later activation is mostly credential/CI work
- document the exact future env vars/keys needed to turn on scanner execution

Do not make Sonar the primary enforcement layer in v1. It should be prepared as a consumer of artifacts and standard repo metadata, not as the place where architecture rules are first encoded.

### Documentation and developer workflow
Add a short current-state analysis doc under `docs/` that replaces stale assumptions and documents:
- which commands developers run
- required one-time installs for external tools
- where reports land
- which boundaries are intentionally enforced for Rust and TS
- which findings are informational vs future blockers

Keep the docs aligned to the actual current repo layout, not the legacy Go-era architecture docs.

## Public Interfaces / Config Additions
Add or update these repo-facing interfaces:
- root npm scripts under `package.json` for `analyze:*`
- `scripts/analysis/*` wrapper scripts
- `dependency-cruiser` config at repo root
- SonarQube config at repo root
- one doc page for analysis workflow and architectural rules

No runtime application APIs should change.

## Test Plan
Validate the integration with these scenarios:
- running each `analyze:*` command with all tools installed produces artifacts in the expected directory
- missing external Rust tools produces a clear install message and a non-crashing exit path
- `dependency-cruiser` flags a deliberate forbidden import between TS layers
- Rust coupling config flags a deliberate forbidden crate dependency
- `analyze:all` runs the Rust and TS analyzers in a predictable sequence and produces an aggregated summary
- SonarQube prep config resolves source paths and excludes vendor/build/report directories correctly

## Assumptions And Defaults
- SonarQube is being prepared locally now, not connected to SonarCloud or a self-hosted server yet.
- New analysis findings are advisory in the first phase and should not block normal development.
- Repo-owned wrapper commands are the primary UX; contributors should not need to remember raw tool invocations.
- Architectural policy is language-specific layer enforcement, not a single cross-stack rule engine.
- Existing legacy docs describing Go/client structure are not authoritative and should not drive the implementation.
