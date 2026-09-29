# TASK-024: ZealotScript — Date Reference

## Context
Zealot is a wiki-plus-planner. Being able to write `@2025-06-01` inline and have it render as
a navigable link to the corresponding planner day (or week/year view) is a key cross-cutting
feature. It keeps references to time contextually anchored in prose.

## Goal
Parse `@YYYY-MM-DD`, `@YYYY-Www` (ISO week), and `@YYYY` inline references and render them as
clickable chips that navigate to the appropriate planner view.

## Requirements

### Syntax variants
| Input | Meaning | Target route |
|---|---|---|
| `@2025-06-01` | Specific day | Planner day view for that date |
| `@2025-W23` | ISO week | Planner week view |
| `@2025` | Year | Planner year view |

### Schema
- New inline atom node: `date_ref` with attrs `{ raw: string, kind: "day" | "week" | "year" }`
- `toDOM`: `["span", { "data-date-ref": raw, class: "zealot-date-ref zealot-date-ref-KIND" }, raw]`

### Viewer behaviour
- In `zealotscript_view.ts`, clicking a `[data-date-ref]` element navigates to the planner
  route for that date/week/year (use the existing router — check how wikilinks navigate)

### Editor display
- Render the raw string inside a pill/chip styled span so it is visually distinct from plain text

### Serializer
- Serialize as the original `@RAW` string

## Implementation Notes
- Parse in `parse_inline.ts` — patterns:
  - Day: `/@(\d{4}-\d{2}-\d{2})\b/`
  - Week: `/@(\d{4}-W\d{2})\b/`
  - Year: `/@(\d{4})\b/` (match last — shortest, would shadow the others)
  - Apply in order: day → week → year
- `kind` attribute is derived from which regex matched
- Parse `@` references **before** general link parsing to avoid conflicts

## Dependencies
- TASK-017 (inline parser)
- TASK-016 (base editor / router)

## Files Likely Involved
- `packages/ui/src/zealotscript/schema.ts` — `date_ref` node spec
- `packages/ui/src/zealotscript/parse/parse_inline.ts` — `@date` patterns
- `packages/ui/src/zealotscript/serializer.ts` — serialize `date_ref`
- `packages/ui/src/zealotscript/zealotscript_view.ts` — click handler
- `packages/content/src/css/` — `.zealot-date-ref` chip styling
