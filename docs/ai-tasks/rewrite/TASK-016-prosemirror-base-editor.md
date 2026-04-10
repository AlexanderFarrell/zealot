# TASK-016: ZealotScript Editor — ProseMirror Base Setup

## Context
The original client used ProseMirror as the rich text editor foundation for item content. The content format is "ZealotScript," a markdown-like format with custom extensions.

Run `git diff master -- client/src/features/zealotscript/zealotscript_editor.ts` and `git diff master -- client/src/features/zealotscript/schema.ts` to see the originals.

## Goal
Set up the ProseMirror editor with the ZealotScript schema as the foundation for TASK-017–020.

## Requirements
- Install ProseMirror packages: `prosemirror-state`, `prosemirror-view`, `prosemirror-model`, `prosemirror-commands`, `prosemirror-keymap`, `prosemirror-history`
- Define the ZealotScript ProseMirror schema (nodes and marks):
  - Nodes: `doc`, `paragraph`, `heading`, `code_block`, `bullet_list`, `ordered_list`, `list_item`, `table`, `table_row`, `table_cell`, `admonition`, `hard_break`
  - Marks: `bold`, `italic`, `code`, `link`
- Create `ZealotScriptEditor` custom element that:
  - Accepts a `content: string` property (raw ZealotScript markdown)
  - Renders a ProseMirror editor view
  - Emits a `change` event with the updated content string on each edit
- Wire basic keyboard shortcuts: bold (`Cmd+B`), italic (`Cmd+I`), undo/redo
- Toolbar with bold, italic, code, link buttons

## Dependencies
None — this is the foundation for TASK-017–020.

## Files Likely Involved
- `packages/ui/src/` (new `zealotscript_editor.ts`, `zealotscript_schema.ts`)
