# TASK-026: ZealotScript — Wikilink QoL (Horizontal Rule, Heading Anchors, Item Alias)

## Context
Three small but high-frequency features that improve the linking experience and document
structure. They are grouped here because they are each low-effort and touch overlapping parts of
the parser and serializer.

---

## Feature A: Horizontal Rule

### Syntax
`---` on its own line (three or more hyphens, optional surrounding whitespace).

### Requirements
- Parse into ProseMirror `horizontal_rule` node (already in `prosemirror-schema-basic`)
- Ensure the parser checks for `---` **before** it falls through to a paragraph
- Serializer: emit `---`

---

## Feature B: Heading Anchors

### Syntax
```
[[item#Section Title]]
```
Links to a specific heading within another item. The part before `#` is the item name/id; the
part after `#` is the heading text to scroll to.

### Requirements
- Parser: extend the existing wikilink regex in `parse_inline.ts` to capture an optional
  `#anchor` segment: `\[\[([^\]#|]+)(?:#([^\]|]+))?(?:\|([^\]]+))?\]\]`
  - Group 1: item target
  - Group 2: anchor (optional)
  - Group 3: display label (optional, see Feature C)
- The `wikilink` inline node (or mark) gains an optional `anchor: string` attribute
- `toDOM`: `<a href="/item/ID#anchor" …>` — the anchor portion is appended to the href
- In the viewer, clicking navigates to the item and then scrolls to the matching `<h*>` element
  by matching its text content against the anchor value (case-insensitive, trim whitespace)
- Serializer: emit `[[target#anchor]]` or `[[target#anchor|label]]`

---

## Feature C: Item Alias (Display Label)

### Syntax
```
[[long item title|short label]]
```
The display text differs from the link target.

### Requirements
- Extend the same wikilink regex (see Feature B) to capture the optional `|label` segment
- The `wikilink` node gains an optional `label: string` attribute
- `toDOM`: render `label` as the link text instead of the item name
- Serializer: emit `[[target|label]]` or `[[target#anchor|label]]`
- Combination: `[[target#anchor|label]]` is valid

---

## Implementation Notes
- The three features share a single regex change in `parse_inline.ts`
- `horizontal_rule` is already in the base schema from `prosemirror-schema-basic` — just ensure
  the parser recognizes `---` before it reaches the paragraph fallback
- For heading scroll: after navigation, use `document.querySelector` to find a heading whose
  `textContent` matches the anchor; call `scrollIntoView`

## Dependencies
- TASK-017 (parser / inline parser)
- TASK-018 (serializer)

## Files Likely Involved
- `packages/ui/src/zealotscript/parser.ts` — add `---` check in main loop
- `packages/ui/src/zealotscript/parse/parse_inline.ts` — updated wikilink regex
- `packages/ui/src/zealotscript/schema.ts` — update wikilink node/mark attrs
- `packages/ui/src/zealotscript/serializer.ts` — emit `---`, `[[…#…|…]]`
- `packages/ui/src/zealotscript/zealotscript_view.ts` — anchor scroll on click
