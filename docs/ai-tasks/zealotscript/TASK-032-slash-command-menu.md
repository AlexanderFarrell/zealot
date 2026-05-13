# TASK-032: ZealotScript — Slash Command Menu

## Context
The slash command menu (`/`) is a standard pattern in modern wiki editors (Notion, Coda, Linear)
that lets users insert any block type by name without reaching for the toolbar. It dramatically
speeds up writing and is especially important on mobile where toolbars are cramped.

## Goal
When the user types `/` at the start of an empty block or after a space, open a searchable
command palette; selecting an entry inserts the corresponding block.

## Requirements

### Trigger
- `/` typed at the start of an empty paragraph **or** at the beginning of any line (after the
  user is at position 0 of the text cursor in that block)
- Popup appears immediately after the `/` character is typed

### Popup UI
- Floating popup anchored just below the cursor position
- Text input for filtering (`/tab`, `/math`, etc.)
- Keyboard navigation: `↑` / `↓` to move selection, `Enter` to confirm, `Escape` to close
- Mouse click also confirms selection
- If the filter produces no results, show "No results"

### Command list (minimum set to implement)
| Label | Inserted block |
|---|---|
| Heading 1–6 | `<h1>`–`<h6>` node |
| Bullet list | `bullet_list` with one empty item |
| Ordered list | `ordered_list` with one empty item |
| Task list | `bullet_list` with one unchecked `list_item` (requires TASK-025) |
| Code block | `code_block` with empty language |
| Table | 2×2 table (reuse `insertTable()` from `commands.ts`) |
| Blockquote | `blockquote` with empty paragraph |
| Admonition (note/warning/tip/info) | `admonition` node (reuse `insertAdmonition()`) |
| Horizontal rule | `horizontal_rule` (requires TASK-026) |
| Math block | `math_block` (requires TASK-021) |
| Mermaid | ` ```mermaid ``` ` code block (requires TASK-027) |
| Details (collapsible) | `details` node (requires TASK-028) |
| Spoiler | `spoiler` node (requires TASK-028) |
| Columns | `columns` with 2 empty `column` nodes (requires TASK-028) |
| Tabs | `tabs` with 2 empty `tab` nodes (requires TASK-028) |
| Timeline | empty `timeline` (requires TASK-030) |
| Progress | `progress` block at 0/100 (requires TASK-030) |
| Map pin | `map_pin` with placeholder coords (requires TASK-031) |
| YouTube embed | `youtube_embed` with prompt for URL |

### Behaviour after selection
- Remove the typed `/` + any filter text from the document
- Insert the chosen node at the cursor position
- Move the cursor inside the new node (e.g. into the first editable cell)

## Implementation Notes
- Implement as a ProseMirror **plugin** with a `EditorView.decorations` pass that opens the
  popup when a `/` is detected at the right position
- Track the slash position and current filter text in plugin state
- The popup itself is a regular DOM element (`document.body` child, absolutely positioned)
  updated on each `decorations` call
- Close the popup on `Escape`, on any transaction that moves the cursor away from the `/` position,
  or on document blur
- Register commands as an array of `{ label, keywords, icon?, insert(state, dispatch) }` objects
  so new block types from other tickets can register themselves without modifying the core plugin

## Dependencies
- TASK-016 (ProseMirror editor base)
- TASK-020 (existing commands)
- All block-type tickets (TASK-021 through TASK-031) for full command list

## Files Likely Involved
- `packages/ui/src/zealotscript/slash_menu.ts` — new file: plugin + popup logic
- `packages/ui/src/zealotscript/zealotscript_editor.ts` — add plugin to editor state
- `packages/ui/src/zealotscript/commands.ts` — export command descriptors for reuse
- `packages/content/src/css/` — `.zealot-slash-menu` popup styling
