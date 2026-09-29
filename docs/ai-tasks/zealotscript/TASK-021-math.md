# TASK-021: ZealotScript — Math (Inline & Block)

## Context
Personal wikis and technical notes frequently contain mathematical expressions. ZealotScript has
no math support today. This ticket adds both inline math (`$…$`) and block math (`:::math`),
rendered via **KaTeX** (preferred over MathJax for its speed and bundle size).

## Goal
Render LaTeX math expressions — both inline within a paragraph and as standalone display blocks.

## Requirements

### Inline math
- Syntax: `$E = mc^2$`
- A single `$` on each side; no space between `$` and content allowed (e.g. `$ E $` is not math)
- Parsed as a new inline mark or atom node: `math_inline`
- Rendered as `<span class="zealot-math-inline">` with KaTeX output injected
- Serialized back to `$…$`

### Block math
- Syntax:
  ```
  :::math
  \sum_{i=1}^{n} i = \frac{n(n+1)}{2}
  :::
  ```
- Parsed as a new block node: `math_block`
- Rendered as `<div class="zealot-math-block">` with KaTeX display-mode output
- Serialized back to `:::math\n…\n:::`
- Register `math` in `ADMONITION_KINDS` (or handle before the admonition parser so it gets its
  own dedicated node type rather than an admonition wrapper)

### Editor behaviour
- In the editor, show the raw LaTeX source inside the node (do not attempt live preview in edit
  mode — keep it simple)
- In the viewer (`zealotscript_view.ts`), render KaTeX after the ProseMirror doc is mounted

### Error handling
- If KaTeX throws (invalid LaTeX), render the raw source in a red `<span class="zealot-math-error">`

## Implementation Notes
- Install KaTeX: `npm install katex` in `packages/ui`; import types from `@types/katex`
- KaTeX render call: `katex.renderToString(source, { displayMode: false, throwOnError: false })`
- The `math_block` node is `atom: true` in the schema (its content is opaque LaTeX text stored
  in an attribute, not as ProseMirror inline content)
- Similarly `math_inline` should be an inline atom node with a `src` attribute
- Parse inline math **before** bold/italic rules in `parse_inline.ts` to avoid `*` conflicts

## Dependencies
- TASK-016 (ProseMirror base editor)
- TASK-017 (parser infrastructure)

## Files Likely Involved
- `packages/ui/src/zealotscript/schema.ts` — add `math_inline`, `math_block` node specs
- `packages/ui/src/zealotscript/parse/parse_inline.ts` — inline `$…$` rule
- `packages/ui/src/zealotscript/parse/parse_math_block.ts` — new file
- `packages/ui/src/zealotscript/parser.ts` — register `parse_math_block` in `multiblockTypes`
- `packages/ui/src/zealotscript/serializer.ts` — serialize both node types
- `packages/ui/src/zealotscript/zealotscript_view.ts` — KaTeX render pass after mount
- `packages/content/src/css/` — `.zealot-math-block`, `.zealot-math-inline`, `.zealot-math-error`
