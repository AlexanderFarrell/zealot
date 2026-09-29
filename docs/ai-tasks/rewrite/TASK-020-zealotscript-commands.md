# TASK-020: ZealotScript — Custom Commands (Admonitions, Tables, Type References)

## Context
The original ZealotScript editor had three custom slash-commands for inserting special block types.

Run `git diff master -- client/src/features/zealotscript/commands/` to see all three command implementations.

## Goal
Implement the three custom editor commands in the new ZealotScript editor.

## Commands

### 1. Admonition Block
- Run `git diff master -- client/src/features/zealotscript/commands/admonition.ts`
- Inserts a callout/admonition block (e.g. note, warning, tip)
- Has a type selector (note / warning / tip / info) and a content area

### 2. Table
- Run `git diff master -- client/src/features/zealotscript/commands/table.ts`
- Also see `git diff master -- client/src/features/zealotscript/table_utils.ts`
- Inserts a new table with configurable rows × columns
- Table cell navigation with Tab key
- Add/remove rows and columns via context menu or toolbar

### 3. Type References
- Run `git diff master -- client/src/features/zealotscript/commands/types.ts`
- Allows inline reference to an item type
- Opens the inline item search (TASK-015) to pick a type
- Rendered as a styled inline chip/badge

## Implementation Notes
- Commands are triggered via the toolbar or a slash-command palette
- Run `git diff master -- client/src/features/zealotscript/text_commands.ts` for how commands were invoked in the original

## Dependencies
- TASK-016 (base editor)
- TASK-015 (inline search — for type references)

## Files Likely Involved
- `packages/ui/src/zealotscript/commands/`
- `packages/ui/src/zealotscript/table_utils.ts`
