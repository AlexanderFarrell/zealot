# TASK-030: ZealotScript — Timeline & Progress Bar

## Context
Two visual status/tracking block types for personal logs, goal tracking, and project history.
They share the `:::keyword` block syntax and are small enough to implement together.

---

## Feature A: Timeline (`:::timeline`)

### Syntax
```
:::timeline
2025-01-15 | Project kickoff
2025-03-01 | First prototype shipped
2025-06-01 | Beta launch
:::
```

Each line inside the block is: `DATE | Event description`

### Requirements
- Block node: `timeline` — atom (no editable ProseMirror content; raw source in attr)
  - Alternatively: structured content with child nodes `timeline_entry` (`{ date, text }`)
  - Recommendation: use structured child nodes so content is editable in the editor
- Schema:
  - `timeline`: group `block`, content `timeline_entry+`
  - `timeline_entry`: attrs `{ date: string }`, content `inline*`
- `toDOM`:
  ```html
  <div class="zealot-timeline">
    <div class="zealot-timeline-entry">
      <span class="zealot-timeline-date">2025-01-15</span>
      <span class="zealot-timeline-text">Project kickoff</span>
    </div>
    …
  </div>
  ```
- CSS: vertical timeline with connector line (left border + bullet marker)
- Serializer: reconstruct `:::timeline\nDATE | TEXT\n…\n:::`

---

## Feature B: Progress Bar (`:::progress`)

### Syntax variants:
```
:::progress 7/10
:::progress 70%
:::progress 70
```

### Requirements
- Block node: `progress` — atom with attrs `{ value: number, max: number, label: string }`
  - `7/10` → `value=7, max=10`
  - `70%` → `value=70, max=100`
  - `70` (bare number) → `value=70, max=100`
- `toDOM`:
  ```html
  <div class="zealot-progress">
    <div class="zealot-progress-bar" style="width: VALUE%"></div>
    <span class="zealot-progress-label">7 / 10</span>
  </div>
  ```
- Width is clamped to [0, 100]%
- Serializer: prefer `:::progress VALUE/MAX` form; for 100-based use `VALUE%`

---

## Implementation Notes
- `:::progress` is a **single-line** block (no closing `:::`); parse the value from the opening
  line itself, e.g. `:::progress 7/10`
- `:::timeline` uses the standard two-line `:::keyword … :::` fencing
- Timeline date display: render `YYYY-MM-DD` as-is; do not attempt locale formatting (keep the
  source the source of truth)

## Dependencies
- TASK-017 (parser infrastructure)
- TASK-018 (serializer)

## Files Likely Involved
- `packages/ui/src/zealotscript/schema.ts` — `timeline`, `timeline_entry`, `progress` node specs
- `packages/ui/src/zealotscript/parse/parse_timeline.ts` — new file
- `packages/ui/src/zealotscript/parse/parse_progress.ts` — new file
- `packages/ui/src/zealotscript/parser.ts` — register both in `multiblockTypes`
- `packages/ui/src/zealotscript/serializer.ts` — serialize both
- `packages/content/src/css/` — `.zealot-timeline`, `.zealot-progress`
