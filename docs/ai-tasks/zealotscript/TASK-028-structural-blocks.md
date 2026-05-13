# TASK-028: ZealotScript — Structural Blocks (Details, Spoiler, Definition List, Columns, Tabs)

## Context
Long-form wiki pages benefit from structural organization beyond headings and lists. This ticket
covers five related block types that group or conditionally reveal content. They share the
`:::keyword` fencing syntax and can be implemented in a single pass.

---

## Feature A: Collapsible Details (`:::details`)

### Syntax
```
:::details Summary line goes here
Any content — paragraphs, lists, code blocks.
:::
```

### Requirements
- Block node: `details` with attr `{ summary: string }`
- Content: any block nodes
- `toDOM`: `<details class="zealot-details"><summary>SUMMARY</summary><div>…</div></details>`
- Open by default: no (browser default — collapsed)
- Serializer: `:::details SUMMARY\n…\n:::`

---

## Feature B: Spoiler Block (`:::spoiler`)

### Syntax
```
:::spoiler
The butler did it.
:::
```

### Requirements
- Block node: `spoiler` — no summary attribute; the trigger label is always "Show spoiler"
- Content: any block nodes
- `toDOM`: `<details class="zealot-spoiler"><summary>Show spoiler</summary><div>…</div></details>`
- Serializer: `:::spoiler\n…\n:::`

---

## Feature C: Definition List (`:::definition`)

### Syntax
```
:::definition
Term
: The definition of the term.

Another Term
: First sense of the meaning.
: Second sense.
:::
```

### Requirements
- Block node: `definition_list` containing one or more `definition_term` + `definition_desc` pairs
- Schema:
  - `definition_list`: group `block`, content `(definition_term definition_desc+)+`
  - `definition_term`: content `inline*`
  - `definition_desc`: content `block+`
- `toDOM`:
  ```html
  <dl class="zealot-definition-list">
    <dt>Term</dt>
    <dd>Definition</dd>
  </dl>
  ```
- Serializer: reconstruct the `:::definition\n…\n:::` block

---

## Feature D: Columns Layout (`:::columns`)

### Syntax
```
:::columns
:::col
Left column content.
:::
:::col
Right column content.
:::
:::
```

### Requirements
- Outer block node: `columns` — content: `column+`
- Inner block node: `column` — content: `block+`
- Number of columns determined by number of `:::col` children (2–4)
- `toDOM`:
  ```html
  <div class="zealot-columns zealot-columns-N">
    <div class="zealot-column">…</div>
    <div class="zealot-column">…</div>
  </div>
  ```
- CSS: `display: grid; grid-template-columns: repeat(N, 1fr)`
- Serializer: reconstruct nested `:::columns / :::col` blocks

---

## Feature E: Tabs (`:::tabs`)

### Syntax
```
:::tabs
:::tab Installation
npm install zealot
:::
:::tab Usage
Import and run.
:::
:::
```

### Requirements
- Outer block node: `tabs` — content: `tab+`
- Inner block node: `tab` with attr `{ title: string }` — content: `block+`
- `toDOM`:
  ```html
  <div class="zealot-tabs">
    <div class="zealot-tab-bar">
      <button class="zealot-tab-btn" data-tab="0">Installation</button>
      <button class="zealot-tab-btn" data-tab="1">Usage</button>
    </div>
    <div class="zealot-tab-panel" data-panel="0">…</div>
    <div class="zealot-tab-panel" data-panel="1">…</div>
  </div>
  ```
- Default: first tab active
- Tab switching: click event listener in `zealotscript_view.ts` (toggle `hidden` attribute on panels)
- Serializer: reconstruct `:::tabs / :::tab TITLE` blocks

---

## Implementation Notes
- All five use the `:::keyword` syntax; extend the admonition-style parser in `parser.ts`
  (or factor into dedicated `parse_structural_blocks.ts`)
- Parse before admonitions so `details`, `spoiler`, `columns`, `tabs`, `col`, `tab`,
  `definition` are not caught by the generic admonition handler
- `definition` list inner syntax (bare lines + `: ` prefix) needs a dedicated mini-parser
- The nested `:::col` / `:::tab` parsing requires a recursive block parser that re-enters the
  main block parsing loop for inner content

## Dependencies
- TASK-017 (parser infrastructure)
- TASK-018 (serializer)

## Files Likely Involved
- `packages/ui/src/zealotscript/schema.ts` — 7 new node specs
- `packages/ui/src/zealotscript/parse/parse_structural_blocks.ts` — new file
- `packages/ui/src/zealotscript/parser.ts` — register in `multiblockTypes`
- `packages/ui/src/zealotscript/serializer.ts` — serialize all five types
- `packages/ui/src/zealotscript/zealotscript_view.ts` — tab click handler
- `packages/content/src/css/` — styles for all five block types
