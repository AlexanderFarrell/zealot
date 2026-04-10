# TASK-018: ZealotScript Serializer

## Context
The original client had a serializer that converts a ProseMirror document back to a ZealotScript markdown string. This is needed to save edits back to the server.

Run `git diff master -- client/src/features/zealotscript/serializer.ts` to see the original.

## Goal
Implement the ProseMirror document → ZealotScript string serializer.

## Requirements
Serialize all node and mark types defined in the schema (TASK-016) back to ZealotScript markdown:

- `paragraph` → plain text line
- `heading` → `# text`, `## text`, etc.
- `bullet_list` / `ordered_list` / `list_item` → markdown list syntax
- `code_block` → fenced code block with language tag
- `table` / `table_row` / `table_cell` → pipe-delimited table syntax
- `admonition` → custom ZealotScript admonition syntax (check original)
- `bold` mark → `**text**`
- `italic` mark → `*text*`
- `code` mark → `` `text` ``
- `link` mark → `[text](url)`

## Implementation Notes
- Serializer takes a ProseMirror `Node` (document) → returns `string`
- Round-trip test: `serialize(parse(s)) === s` for all valid ZealotScript inputs
- Handle edge cases: nested lists, table cells with inline marks, empty paragraphs

## Dependencies
- TASK-016 (schema)
- TASK-017 (parser — needed to write round-trip tests)

## Files Likely Involved
- `packages/ui/src/zealotscript/serializer.ts`
