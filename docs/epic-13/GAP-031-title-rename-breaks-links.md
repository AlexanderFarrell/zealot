# GAP-031 — Renaming an item title silently breaks all inbound wiki links

## Problem

`crates/zealot-app/src/services/item.rs` `update_item()` does not re-scan other items'
content when a title changes. If item "Fix login bug" is renamed to "Fix auth bug",
every other item containing `[[Fix login bug]]` now has a broken wiki link.

The link record for "Fix login bug" → (old title) is removed when `rebuild_links` next
runs, but other items' content still contains the literal string `[[Fix login bug]]`,
which now resolves to nothing.

There is no reverse index — no efficient way to know which items reference a given
title — so fixing this requires a full-account content scan on every title change.

## Impact

Title changes are common during normal use (renaming tasks, correcting typos, evolving
project names). Every rename silently breaks the network of connections that makes a
wiki valuable. Users discover this when clicking a link that produces no navigation.

## Proposed fix — Option A (recommended): Trigger full rebuild on title change

In `update_item()`, detect when `title` has changed. If changed, call
`rebuild_links_for_account()` asynchronously after the update succeeds.

This is O(n) items per rename, which is acceptable for personal-scale instances. For
large accounts it may be slow — see Option B.

## Proposed fix — Option B: Maintain a reverse title index

Store a `wiki_link` table (`from_item_id`, `to_title`) updated on every save. On title
change, scan the reverse index to find items that reference the old title and update
their content. More efficient but significantly more complex.

## Proposed fix — Option C: Document the limitation clearly

At minimum, note in `docs/http-api.md`, `docs/data-model.md`, and the quickstart that
title changes require a manual `POST /item/rebuild-links` to restore wiki link integrity.
Add a warning in the item rename UI.

Option A is the right balance for Zealot's scale. Option C is the minimum to do now.

## Files to change

- `crates/zealot-app/src/services/item.rs` — detect title change, trigger rebuild
- `docs/data-model.md` — document wiki link behaviour on title changes
- `docs/http-api.md` — add note to wiki link and rebuild-links sections
