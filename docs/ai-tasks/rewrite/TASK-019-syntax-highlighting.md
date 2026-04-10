# TASK-019: ZealotScript — Code Block Syntax Highlighting

## Context
The original client applied syntax highlighting to fenced code blocks in the rendered output. Language was inferred from the code fence tag.

Run `git diff master -- client/src/features/zealotscript/parse/parse_code_block.ts` to see the original approach and which highlighting library was used.

## Goal
Add syntax highlighting to code blocks in the ZealotScript editor.

## Languages to Support
bash, C, C++, CSS, Go, JavaScript, JSON, Markdown, Python, Rust, SQL, TypeScript, XML, YAML

## Requirements
- Detect the language tag on the code fence (e.g. ` ```rust `)
- Apply token-based syntax highlighting within the code block in the ProseMirror view
- Use a lightweight highlighting library (check original — likely highlight.js or Prism; use the same one if possible)
- Highlighting should be purely visual (decorations) and not affect the underlying document content
- Fallback: no highlighting if language is unknown or missing

## Dependencies
- TASK-016 (base editor)
- TASK-017 (parser — code blocks must be parsed first)

## Files Likely Involved
- `packages/ui/src/zealotscript/parse/parse_code_block.ts`
- `packages/ui/src/zealotscript/zealotscript_editor.ts`
