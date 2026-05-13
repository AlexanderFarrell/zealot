# TASK-029: ZealotScript — Transclusion & Hover Preview

## Context
Transclusion — embedding one item's content inside another — is the defining feature of a wiki
vs. a flat text editor. Hover preview is a companion UX that lets the user peek at a linked item
without navigating away. Both features depend on the API returning item content on demand.

---

## Feature A: Transclusion (`![[item name]]`)

### Syntax
```
See also the full background:

![[Project Background]]
```

### Requirements
- Parsed as a new block atom node: `transclusion` with attr `{ target: string }` (item name or id)
- `target` is the text between `![[` and `]]`
- In the **viewer**: fetch the target item's content via the API, parse it as ZealotScript, and
  render the resulting subtree inside a `<div class="zealot-transclusion">` wrapper
  - Show a loading state while fetching
  - Show a "not found" placeholder if the item does not exist
  - Render inline (not in an iframe)
  - Transcluded content is **read-only** (no nested editor)
  - Infinite transclusion loops must be detected and broken — track a depth counter or a set of
    already-transcluded item IDs passed down from the host viewer; stop and show a warning at
    depth > 3 or on a cycle
- In the **editor**: render the node as a non-editable chip showing the target name:
  `⤵ Project Background` with a link icon; actual content expansion only happens in the viewer
- Serializer: `![[target]]`

### API
- Use an existing API client from `packages/api/src/` to fetch item content by name/id
- If no such method exists, add `getItemByName(name: string): Promise<Item>` to the appropriate
  API client

---

## Feature B: Hover Preview for Wikilinks

### Requirements
- When the user hovers over a `[[wikilink]]` in the viewer (and stays for ~400 ms), show a
  floating preview popup containing:
  - The item's title
  - The first ~200 characters of its content rendered as ZealotScript
- Popup dismisses when the pointer leaves the link or the popup
- Popup positioning: prefer appearing above or below the link, within the viewport
- Reuse the existing popup/tooltip infrastructure from `packages/engine/src/` if one exists;
  otherwise build a simple absolutely-positioned `<div>` managed in `zealotscript_view.ts`
- API: same endpoint as transclusion — fetch by item name

---

## Implementation Notes
- Transclusion fetch should be cached for the lifetime of the viewer render to avoid redundant
  API calls when the same item is transcluded more than once on a page
- Hover preview debounce: use a `setTimeout` of 400 ms; cancel on `mouseleave`
- Do not implement hover preview inside the editor (too noisy while writing)

## Dependencies
- TASK-017 (parser)
- TASK-018 (serializer)
- TASK-026 (wikilink node — hover attaches to the same anchor elements)

## Files Likely Involved
- `packages/ui/src/zealotscript/schema.ts` — `transclusion` node spec
- `packages/ui/src/zealotscript/parse/parse_transclusion.ts` — new file: detect `![[…]]`
- `packages/ui/src/zealotscript/parser.ts` — register transclusion parser (before wikilinks)
- `packages/ui/src/zealotscript/serializer.ts` — emit `![[target]]`
- `packages/ui/src/zealotscript/zealotscript_view.ts` — fetch + render transclusion; hover logic
- `packages/api/src/` — add `getItemByName` if missing
- `packages/content/src/css/` — `.zealot-transclusion`, `.zealot-hover-preview`
