# TASK-033: ZealotScript — Word Count & Reading Time in Toolbar

## Context
A simple word/character count and estimated reading time helps writers calibrate the length of
wiki entries, meeting notes, and longer posts without leaving the editor.

## Goal
Display a live "N words · N min read" indicator in the editor toolbar that updates as the user
types.

## Requirements

### Counts to display
- **Word count**: number of whitespace-separated words across all text nodes in the document
  (exclude code block contents — those are not prose)
- **Character count**: total character count including spaces (exclude code block contents)
- **Reading time**: `Math.ceil(wordCount / 200)` minutes (200 wpm average)

### Display format
```
247 words · 1 min read
```
- Position: right side of the toolbar (or below the toolbar in a status bar row)
- Updates on every document change (debounced 300 ms to avoid layout thrash)
- Hidden when the editor is empty (0 words)

### What counts as a word
- Split on `/\s+/` after stripping all inline marks
- Code spans (`` `…` ``) and code blocks are excluded
- Wikilinks contribute their display label (or target name) to the count
- Math nodes: excluded

## Implementation Notes
- Walk the ProseMirror document tree with `doc.descendants(node => { … })` to collect all text
  node content, skipping nodes inside `code_block`
- Hook into the ProseMirror `EditorView` update cycle: override `dispatchTransaction` or use a
  plugin with an `update` callback to recalculate on each state change
- The debounce can be a simple `setTimeout` / `clearTimeout` pattern in the plugin's `update`
- Render into a `<span class="zealot-word-count">` injected into the existing toolbar element
  in `zealotscript_editor.ts`

## Dependencies
- TASK-016 (editor)

## Files Likely Involved
- `packages/ui/src/zealotscript/zealotscript_editor.ts` — word count plugin + toolbar span
- `packages/content/src/css/` — `.zealot-word-count` positioning/styling
