# TASK-001: Navigator Interface + Web Router

## Context
The new client (`apps/web/`) has no router. The original client (`master:client/`) uses Navigo for client-side routing.

Navigation is a platform-specific concern: the web app uses URL-based routing; a future desktop app will use tabs; mobile may have its own model. To avoid coupling screens to a web-only abstraction, we split this into two pieces:

1. A `Navigator` interface in `packages/engine` — the universal "open this page" contract
2. A `WebNavigator` in `apps/web/` — the web implementation (URL routing, history, DOM swap)

Run `git diff master -- client/src/features/router/` to see the original router implementation.

## Part 1 — Navigator Interface (`packages/engine`)

Create `packages/engine/src/ui/navigator.ts` and export it from `packages/engine/src/index.ts`.

```ts
export type SettingsSection = 'attributes' | 'types' | 'planner' | 'wiki' | 'data' | 'user';
export type PlannerView = 'daily' | 'weekly' | 'monthly' | 'annual';

export interface Navigator {
  openHome(): void;
  openItem(title: string): void;
  openItemById(id: number): void;
  openMedia(path: string): void;
  openPlanner(view: PlannerView, date?: string): void;
  openTypes(): void;
  openType(title: string): void;
  openAnalysis(): void;
  openRules(): void;
  openSettings(section?: SettingsSection): void;
}
```

Also add a `setNavigator` / `getNavigator` module-level accessor in that file so any component can call `getNavigator().openItem(title)` without prop-drilling. The accessor should throw clearly if called before a navigator has been registered.

## Part 2 — WebNavigator (`apps/web`)

Create `apps/web/src/web_navigator.ts` that implements `Navigator` using a hand-rolled history router (or Navigo — match the original). Responsibilities:

- Implement every `Navigator` method by calling `history.pushState` + rendering the appropriate screen into `<content->`.
- Register a `popstate` listener to handle back/forward.
- Call `setNavigator(new WebNavigator())` during app startup (in `apps/web/src/web_client.ts` or `core.ts`).

## Routes to implement

| Navigator method | Path pushed | Screen |
|---|---|---|
| `openHome()` | `/` | ItemScreen (loads "Home" item) |
| `openItem(title)` | `/item/:title` | ItemScreen (by title) |
| `openItemById(id)` | `/item_id/:id` | ItemScreen (by ID) |
| `openMedia(path)` | `/media/*` | MediaScreen |
| `openPlanner('daily', date?)` | `/planner/daily/:date` | DailyPlannerScreen (defaults to today) |
| `openPlanner('weekly', date?)` | `/planner/weekly/:date` | WeeklyPlannerScreen |
| `openPlanner('monthly', date?)` | `/planner/monthly/:date` | MonthlyPlannerScreen |
| `openPlanner('annual', date?)` | `/planner/annual/:year` | AnnualPlannerScreen |
| `openTypes()` | `/types` | TypesScreen |
| `openType(title)` | `/types/:title` | TypeScreen |
| `openAnalysis()` | `/analysis` | AnalysisScreen |
| `openRules()` | `/rules` | RulesScreen |
| `openSettings(section?)` | `/settings/:section` | SettingsScreen (defaults to `attributes`) |

## Implementation Notes
- The `WebNavigator` also needs a `resolve()` method (or equivalent) called once at startup to parse the current URL and render the right screen — this handles direct links and page refreshes.
- Handle 404 / unknown routes gracefully.
- Handle 401 responses globally (in `packages/engine/src/api/api_helper.ts` or similar): clear the stored account in AuthAPI and call `getNavigator().openSettings('user')` or redirect to login.
- The command system (`packages/engine/src/ui/commands.ts`) should register navigation commands (e.g. "Go to Home", "Open Planner") that delegate to `getNavigator()`.

## Files Likely Involved
- `packages/engine/src/ui/navigator.ts` ← new
- `packages/engine/src/index.ts` ← add export
- `apps/web/src/web_navigator.ts` ← new
- `apps/web/src/web_client.ts`
- `apps/web/src/core.ts`
