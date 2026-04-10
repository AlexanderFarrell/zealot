# Zealot — Claude Code Instructions

## Project Overview
Zealot is a personal wiki + planner application. The backend is Rust (Axum), the frontend is a TypeScript/Web Components client under `apps/web/` with shared packages under `packages/`.

The `master` branch contains the original client under `client/` (vanilla TypeScript, no framework). The current rewrite branches replace it with a structured TypeScript/Web Components architecture.

---

## Rewrite Skill

When asked to work on a rewrite task (e.g. "work on TASK-010" or "implement the item screen"), follow this procedure:

### 1. Read the ticket
Tickets live in `docs/ai-tasks/rewrite/`. Read the relevant file first.

### 2. Study the original
Every ticket instructs you to run a `git diff master -- client/<path>` to read the original implementation. **Always do this.** Do not guess at the original behaviour — read it.

```bash
git diff master -- client/src/features/<feature>/
```

If the ticket references multiple files, diff them all before writing any code.

### 3. Understand the new architecture
Before writing code, read the relevant new files:
- `packages/api/src/` — API clients (HTTP layer)
- `packages/domain/src/` — typed domain objects
- `packages/engine/src/` — shared utilities (events, commands, hotkeys, popups, graphs)
- `packages/ui/src/` — UI components and screens
- `packages/content/src/` — static assets (icons, CSS)
- `apps/web/src/` — entry point and top-level shell

### 4. Follow the component pattern
All UI components are custom elements (`HTMLElement` subclasses) registered via `customElements.define`. See existing components in `packages/ui/src/` for the pattern. Do not introduce a framework (React, Vue, etc.).

### 5. API calls go through `packages/api`
Never make `fetch` calls directly from UI code. Use the typed API clients in `packages/api/src/`. If an endpoint is missing from the API package, add it there first.

### 6. Icons
All icons are SVG exports from `packages/content/src/`. Import from there — do not embed raw SVG in component files.

### 7. Styling
Component-local styles go in a `<style>` tag inside the component's shadow DOM (or as a tagged template). Global/layout styles live in `packages/content/src/`. Do not add `<link>` tags pointing to external CDNs.

### 8. After implementing
- Run `npm run build` (or `tsc --noEmit`) in the relevant package to verify no TypeScript errors
- Cross-check the new implementation against the original diff to ensure nothing was missed
- Mark any intentional deviations from the original in a code comment explaining why

---

## Key Conventions

| Thing | Where it lives |
|---|---|
| HTTP API clients | `packages/api/src/` |
| Domain types / DTOs | `packages/domain/src/` |
| Shared engine utilities | `packages/engine/src/` |
| UI components & screens | `packages/ui/src/` |
| Icons, CSS assets | `packages/content/src/` |
| App entry point | `apps/web/src/main.ts` |
| Rewrite task tickets | `docs/ai-tasks/rewrite/` |
| Other AI task docs | `docs/ai-tasks/` |

## Backend Reference
Backend HTTP handlers live in `crates/zealot-api/src/http/`. When verifying API paths or request/response shapes, read the handler there — it is the source of truth.
