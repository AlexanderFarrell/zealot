# GAP-003 — Fix admonition syntax in rules-engine cookbook

## Problem

The rules-engine cookbook's "Building content with ZealotScript" section uses
`!!! note` syntax (Obsidian/MkDocs style) in a Lua string example:

```lua
-- Admonitions (Zealot extension)
"!!! note\n    Body text here\n"
```

But ZealotScript uses `:::note … :::` fenced syntax, not `!!! note`. This is documented
in `data-model.md`, `zealotscript/README.md`, and the quickstart — the `:::` syntax is
what Zealot actually renders.

A user following the cookbook and using `!!! note` will produce a raw text output, not
a rendered callout.

## Fix

In `docs/rules-engine.md`, update the "Building content with ZealotScript → Supported
formatting" section to use correct `:::note … :::` syntax:

```lua
-- Admonitions (Zealot extension)
":::note\nBody text here\n:::\n"
```

Also verify none of the other Lua example strings in the cookbook use `!!! note`.

## Files to change

- `docs/rules-engine.md` — the `!!! note` example at line ~455
