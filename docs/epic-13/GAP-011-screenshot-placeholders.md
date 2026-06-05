# GAP-011 — Replace screenshot placeholders in overview.md

## Problem

`docs/overview.md` contains six screenshot placeholder blocks:

```markdown
> **Screenshot placeholder:** Daily planner view showing a mix of scheduled tasks…
> _Replace with `docs/screenshots/planner-daily.png` once captured._
```

Placeholders appear for:
- Daily planner view (`docs/screenshots/planner-daily.png`)
- Item editor with wiki page body (`docs/screenshots/item-wiki-page.png`)
- Item list filtered to project tasks (`docs/screenshots/project-task-list.png`)
- Daily planner repeat section (`docs/screenshots/planner-repeat.png`)
- Types configuration screen (`docs/screenshots/settings-types.png`)
- Rules screen with output log (`docs/screenshots/rules-screen.png`)

These placeholders are visible to anyone reading the documentation and leave the
overview looking unfinished. They also reference a `docs/screenshots/` directory that
does not exist in the repository.

## Fix

Two options:

**Option A — Capture real screenshots:**
Once the UI rewrite is complete enough for the relevant screens, capture real screenshots
and place them in `docs/screenshots/`. Replace each placeholder block with:
```markdown
![Daily planner view](./screenshots/planner-daily.png)
```

**Option B — Remove placeholders until screenshots are ready:**
Remove the placeholder callout blocks from `overview.md`. The prose descriptions above
each placeholder are sufficient for now, and removing the placeholders makes the doc
look clean rather than draft.

Option B is appropriate for the current state of the rewrite. Option A should be done
when the UI is stable.

## Files to change

- `docs/overview.md` — remove or replace the six placeholder blocks
