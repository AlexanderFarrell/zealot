# TASK-037: Rules Screen

## Context
The `/rules` route in the original client was a stub — it rendered only a "Rules" header with no implementation. The backend `RuleAPI` is also a stub. This is lower priority and likely a future feature.

Run `git diff master -- client/src/features/rules/rules_screen.ts` to confirm the extent of the original implementation. Also check the Rust backend for any rule-related handlers to understand if the API exists.

## Goal
Implement the rules/automation editor screen.

## Requirements
- This is intentionally left open-ended until the backend defines the rules schema
- Minimum viable: a screen that loads rules from the API and allows basic CRUD
- Stretch: a visual rule builder (trigger → condition → action)

## Pre-requisites
- The backend must define and implement the rules endpoints before this ticket is actionable
- Check `crates/zealot-api/src/http/` on master for any rule handlers

## Dependencies
- TASK-001 (Router)
- Backend rules implementation

## Files Likely Involved
- `packages/ui/src/` (new `rules_screen.ts`)
- `packages/api/src/rule.ts` (currently a stub — flesh out once backend is ready)
- `packages/domain/src/` (add `Rule` domain type)
