# TASK-025: ZealotScript — Task List (Checklist)

## Context
Task lists are the single highest-value missing feature for a personal wiki-planner. The
standard `- [ ] item` / `- [x] done` syntax is widely understood and integrates naturally with
the existing list parser. Checked tasks should be interactive in both editor and viewer.

## Goal
Parse and render GFM-style task list items; allow toggling the checkbox in the viewer.

## Requirements

### Syntax
```
- [ ] Write the spec
- [x] Ship the parser
- [ ] Add tests
```
- Marker `[ ]` = unchecked, `[x]` or `[X]` = checked
- Task items may appear inside ordered lists too: `1. [ ] First step`
- Nesting is allowed (task inside task): indented by 2 or 4 spaces

### Schema
- Extend the existing `list_item` node with a boolean attribute: `checked: boolean | null`
  - `null` means a regular (non-task) list item
  - `true` / `false` means a task item (checked / unchecked)
- `toDOM`: when `checked !== null`, prepend an `<input type="checkbox">` to the list item

### Viewer behaviour
- Checkboxes are **interactive** in `zealotscript_view.ts`
- Clicking a checkbox toggles `checked` on the node and fires an `item:content-changed` event
  (or equivalent) so the parent component can persist the updated content
- The serializer is called to produce the new ZealotScript string after each toggle

### Editor behaviour
- In the editor, checkboxes render and are clickable (same toggle behaviour)
- Input rule: when `- [ ] ` or `- [x] ` is typed at the start of a line, automatically
  convert it to a task list item
- Toolbar or slash-command: "Insert task list" inserts an unchecked item

### Serializer
- Unchecked: `- [ ] text`
- Checked: `- [x] text`
- Non-task items: `- text` (unchanged)

## Implementation Notes
- Modify `parse_list.ts` — detect `[ ]` / `[x]` prefix after the `- ` or `N. ` marker
- The `list_item` schema update is backward-compatible: existing items have `checked: null`
- `toDOM` for `list_item`: conditionally prepend `["input", { type: "checkbox", checked: "" }]`
  when `checked !== null`
- Use `NodeView` in ProseMirror to make the checkbox interactive in the editor without breaking
  the node's content model

## Dependencies
- TASK-017 (list parser)
- TASK-018 (serializer)

## Files Likely Involved
- `packages/ui/src/zealotscript/schema.ts` — update `list_item` attrs
- `packages/ui/src/zealotscript/parse/parse_list.ts` — detect `[ ]`/`[x]` prefix
- `packages/ui/src/zealotscript/serializer.ts` — emit `- [ ]` / `- [x]`
- `packages/ui/src/zealotscript/zealotscript_editor.ts` — NodeView + input rule
- `packages/ui/src/zealotscript/zealotscript_view.ts` — click handler for checkbox toggle
- `packages/content/src/css/` — task item styling (checked state, strikethrough option)
