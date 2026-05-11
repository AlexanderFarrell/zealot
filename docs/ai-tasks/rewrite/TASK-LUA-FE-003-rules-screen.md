# TASK-LUA-FE-003: Rules screen UI

## Context

`packages/ui/src/screens/rules_screen.ts` exists as a stub (TASK-037 placeholder). This task implements the full rules management screen with a list view and an edit/create view.

Run `git diff master -- client/src/features/rules/rules_screen.ts` to confirm the original was a stub with only a header.

## Goal

Build a fully functional rules management screen with:
1. **List view** — shows all rules with status, enable toggle, run, edit, delete actions
2. **Edit view** — create/edit a rule with trigger selector and Lua script textarea

## Requirements

### List view

- Shows all rules fetched from `RuleAPI.GetAll()`
- Table columns: **Name**, **Trigger**, **Enabled** (toggle), **Last Run**, **Status**, **Actions**
- **Last Run**: formatted relative time (`"2 min ago"`, `"Never"`)
- **Status**: green dot if last run succeeded, red dot if last error is set, grey if never run
- **Actions**: Edit button, Run Now button, Delete button (with confirmation)
- "New Rule" button at top right opens the edit view with an empty form
- Clicking a rule's name also opens the edit view

### Edit view

Fields:
1. **Name** — text input, required
2. **Description** — textarea, optional
3. **Trigger** — `<select>` with all trigger kind options (use `TRIGGER_KIND_LABELS` from domain)
4. **Trigger config** — conditional sub-fields shown based on trigger selection:
   - `cron`: text input for expression with placeholder `"0 9 * * 1"` and hint `"min hour day month weekday"`
   - `interval`: number input for seconds
   - `on_type_assign` / `on_type_unassign`: text input for type_name (optional, blank = any type)
   - `on_attribute_set`: text input for attribute_key (optional, blank = any attribute)
   - All others: no sub-fields
5. **Enabled** — checkbox
6. **Script** — `<textarea>` with `font-family: monospace`, min height 300px, label "Lua Script"
   - Include a link to docs/use-cases in a comment above the textarea
7. Buttons: **Save**, **Cancel**
8. If editing an existing rule: **Run Now** button that calls `RuleAPI.Run(rule_id)` and shows output/error inline

### After "Run Now"

Display result below the form:
- If `result.success`: green panel showing `result.output` (or "Script ran successfully — no output")
- If not `result.success`: red panel showing `result.error`
- Duration displayed as `"ran in 123ms"`

### Component structure

```typescript
// packages/ui/src/screens/rules_screen.ts

export class RulesScreen extends HTMLElement {
    // State: 'list' | 'edit'
    // Current rule being edited (null if creating)
    // Renders list or edit form based on state
}

customElements.define('rules-screen', RulesScreen);
```

Use shadow DOM. Follow the pattern of `ItemScreen` and `TypeScreen`.

### CSS

Add `packages/content/src/css/screens/rules.scss` (or `.css`) with styles for:
- `.rules-list` — table with border-collapse
- `.rules-list td, .rules-list th` — padding, border-bottom
- `.rule-status-dot` — 8px circle, green/red/grey
- `.rule-edit-form` — form layout with label/input pairs
- `.rule-script-area` — monospace textarea
- `.run-result` — green/red panel for run output
- Import this from the main CSS bundle or inline it in the shadow DOM `<style>` tag

## Dependencies

- TASK-LUA-FE-002 (API client)
- TASK-001 (Router — navigate to `/rules`)
- TASK-LUA-012 (backend must exist for real API calls)

## Files to create/modify

- `packages/ui/src/screens/rules_screen.ts`
- `packages/content/src/css/screens/rules.scss` (new)

## Verification

1. Navigate to `/rules` in the browser
2. Verify list loads and shows rules from the API (or empty state)
3. Create a rule with trigger `manual` and script `zealot.notify("test")`
4. Click "Run Now" and verify the output panel shows `"test"`
5. Toggle enabled checkbox, verify PATCH is called
6. Delete a rule with confirmation, verify it disappears from list
