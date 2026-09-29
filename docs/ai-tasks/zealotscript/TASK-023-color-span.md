# TASK-023: ZealotScript — Color Span

## Context
Occasionally a note needs a splash of color for emphasis beyond bold/highlight — e.g. status
labels, priority indicators, or personal color-coding. This adds a minimal inline color mark.

## Goal
Support `<color:red>some text</color>` syntax to apply a foreground color to an inline range.

## Requirements

### Syntax
```
The build is <color:green>passing</color> but deployment is <color:red>blocked</color>.
```

- Opening tag: `<color:VALUE>` where `VALUE` is a CSS color — named color, hex (`#ff0000`),
  or `rgb(…)` / `hsl(…)`
- Closing tag: `</color>`
- Nested color spans are not required; outermost wins if they appear
- Tags may not span block boundaries (inline-only)

### Schema
- New mark: `color` with attribute `{ value: string }`
- `toDOM`: `["span", { style: "color: VALUE", class: "zealot-color" }, 0]`

### Serializer
- Serialize back to `<color:VALUE>…</color>`

### Safety
- In the viewer, the `value` attribute must be sanitized before it is set as a CSS property.
  Accept only: CSS named colors (whitelist), `#` followed by 3 or 6 hex digits, and the
  functional forms `rgb(n,n,n)` / `hsl(n,n%,n%)` matched with a strict regex.
  Anything else renders without any color (mark is a no-op visually).

## Implementation Notes
- Parse in `parse_inline.ts` with a regex that matches `<color:([^>]+)>(.*?)<\/color>`
- Because the tag syntax differs from markdown conventions, parse it in a dedicated pass
  **after** code-span extraction (so color tags inside backticks are not processed)
- The serializer must escape `<` and `>` within the colored text to avoid ambiguity

## Dependencies
- TASK-017 (inline parser)

## Files Likely Involved
- `packages/ui/src/zealotscript/schema.ts` — `color` mark spec
- `packages/ui/src/zealotscript/parse/parse_inline.ts` — `<color:…>` pattern
- `packages/ui/src/zealotscript/serializer.ts` — serialize color mark
- `packages/content/src/css/` — `.zealot-color` (any wrapping style, e.g. border-radius for chips)
