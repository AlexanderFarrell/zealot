# Cross-Platform Navigation and Auxiliary Tool Strategy

## Summary
Keep `Navigator` strictly for canonical destination navigation, and add a second shared interface for auxiliary tools. The three sidebar features in TASK-005/006/007 become reusable tool views, while the web app alone owns the `activePanel` switcher and left-sidebar presentation.

## Shared Contracts
- Expand `Navigator` so it is readable as well as writeable:
  - keep the existing `openHome/openItem/openPlanner/...` methods
  - add `getLocation(): AppLocation`
  - add `subscribe(listener): () => void`
- Define `AppLocation` as a typed union for all canonical destinations:
  - `home`
  - `item` with `itemId?: number`, `title?: string`
  - `planner` with `view` and `date`
  - `media`
  - `types`
  - `type`
  - `analysis`
  - `rules`
  - `settings`
  - `not_found`
- Add a new shared shell/tool contract beside `Navigator`, not inside it:
  - `ToolView = 'search' | 'nav_tree' | 'calendar' | 'item_attributes'`
  - `ToolHost.show(view, options?: { focus?: boolean }): void`
  - `setToolHost/getToolHost` accessors
- Keep tool-opening semantic, not layout-specific:
  - no `openSidebarPanel`
  - no shared `activePanel`
  - no sidebar terminology in engine APIs

## Platform Strategy
- Web:
  - implements `Navigator` with URL/history as today
  - implements `ToolHost` with a local `activePanel`
  - owns the left-rail switcher and sidebar host in `apps/web`
  - does not put active panel state in the URL
- Desktop:
  - implements the same `ToolHost`, but may open a tool in a dock, side tab, or regular tab
  - tabs remain a desktop presentation choice, not part of the shared contract
- Mobile:
  - implements the same `ToolHost`, but may open the tool as a full-screen screen, drawer, or sheet
  - this still satisfies `show('search')` without introducing web-only concepts

## TASK-005/006/007 Implications
- Build nav tree, calendar, and search as reusable tool views in shared UI.
- Move the panel-switching host out of shared UI and into the web app layer.
- Use `Navigator.subscribe()` for cross-platform “current location” state such as nav-tree active-item highlighting.
- Use `ToolHost` for commands like `Search Items`, `Open Nav Sidebar`, and `Open Calendar`.
- Keep TASK-008 out of implementation, but reserve the `item_attributes` tool id now so the contract does not change later.

## Test and Acceptance
- Web still supports left-rail switching among nav tree, calendar, and search.
- Nav tree highlight derives from `Navigator` location state, not web-only shell state.
- Search/calendar open through `ToolHost` commands, not direct sidebar mutation.
- No shared module assumes sidebar, tabs, or full-screen presentation.
- Direct `tsc` validation remains the verification path until the empty `packages/core/package.json` repo issue is fixed.

## Assumptions and Refactor Notes
- `packages/ui` should own reusable tool views, not the web-specific switcher.
- `apps/web` should own sidebar orchestration and panel persistence.
- The current command-name drift should still be cleaned up as part of this work.
- If later needed, a desktop app can map `show('search')` to “open/focus Search tab” without changing shared UI or engine contracts.
