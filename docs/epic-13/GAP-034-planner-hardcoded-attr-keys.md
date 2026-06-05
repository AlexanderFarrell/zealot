# GAP-034 — Planner attribute keys are hardcoded magic strings

## Problem

`crates/zealot-app/src/services/planner.rs` hardcodes the strings `"Date"`, `"Week"`,
`"Month"`, and `"Year"` as the attribute keys the planner filters on (lines 38, 54,
72-78, 95).

These strings are:
1. **Case-sensitive.** A user who creates an attribute kind named `"date"` (lowercase)
   instead of `"Date"` sees an empty planner with no error. There is zero feedback.
2. **Not configurable.** There is no way to point the planner at different attribute
   keys (e.g., `"Due Date"` instead of `"Date"`).
3. **Not validated at setup.** On a fresh instance with no attribute kinds, the planner
   silently returns empty results.

## Impact

This is the most likely source of "the planner is empty and I don't know why" support
issues. The quickstart tells users to create a `Date` attribute kind — but if they
create it with different casing or naming, the planner never works.

## Proposed fix — Option A (recommended): Case-insensitive matching in the query

Change the planner SQL queries to use `LOWER(key) = 'date'` instead of `key = 'Date'`.
This tolerates the most common mistake (wrong case) with no config needed.

## Proposed fix — Option B: Configurable planner attribute keys per account

Add account-level settings (`planner_date_key`, `planner_week_key`, etc.) that default
to `"Date"`, `"Week"`, etc. Expose in the account settings UI and API.

## Proposed fix — Option C: First-run validation warning

On first login (or on the settings screen), check whether the standard attribute kinds
exist. If `"Date"` is missing, show a prominent warning: "The planner needs a Date
attribute kind to show items. Create it in Settings → Attributes."

Option A is the easiest win. Option C is the right UX companion.

## Files to change

- `crates/zealot-app/src/services/planner.rs` — change to case-insensitive key matching
- (For Option C) Frontend settings screen — add validation warning
