# TASK-017: ZealotScript Parser

## Context
The original client had a custom markdown parser that converts ZealotScript strings into ProseMirror documents. This is needed to load existing item content into the editor.

Run `git diff master -- client/src/features/zealotscript/parser.ts` and `git diff master -- client/src/features/zealotscript/parse/` to see the originals.

## Goal
Implement the ZealotScript → ProseMirror document parser.

## Requirements
Parse the following ZealotScript constructs into ProseMirror nodes:

### Block-level
- Paragraphs (run `git diff master -- client/src/features/zealotscript/parse/parse_paragraph.ts`)
- Headings (`# H1`, `## H2`, `### H3`)
- Ordered lists (`1. item`) and unordered lists (`- item`) — run `git diff master -- client/src/features/zealotscript/parse/parse_list.ts`
- Fenced code blocks (`` ``` ``) with optional language tag — run `git diff master -- client/src/features/zealotscript/parse/parse_code_block.ts`
- Tables — run `git diff master -- client/src/features/zealotscript/parse/parse_table.ts`
- Admonition blocks (custom ZealotScript syntax — check the original)

### Inline
- Bold (`**text**`)
- Italic (`*text*` or `_text_`)
- Inline code (`` `code` ``)
- Links (`[text](url)`)
- Run `git diff master -- client/src/features/zealotscript/parse/parse_inline.ts` for all inline rules

## Implementation Notes
- Parser takes a `string` → returns a ProseMirror `Node` (document)
- Should be robust to malformed input — unknown syntax passes through as plain text
- Build incrementally: paragraphs + inline first, then blocks, then tables/admonitions

## Dependencies
- TASK-016 (ProseMirror schema must exist first)

## Files Likely Involved
- `packages/ui/src/zealotscript/parser.ts`
- `packages/ui/src/zealotscript/parse/` (one file per construct type)
