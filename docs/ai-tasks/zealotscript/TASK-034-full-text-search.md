# TASK-034: ZealotScript — Full-Text Search Across All Items

## Context
Full-text search is the most important navigation tool once a wiki grows beyond a handful of
items. The existing search sidebar (TASK-007) likely searches item titles only. This ticket
extends it to search the full body content of every item, surfacing matching headings and
context snippets.

## Goal
Provide a search experience that queries the full text of all item content and returns results
with a snippet highlighting the match context and the nearest heading anchor.

## Requirements

### Search input
- Reuse the existing search sidebar UI from TASK-007
- A new "Full text" toggle or mode switch distinguishes full-text from title-only search
- Search triggers on every keystroke (debounced 300 ms) or on Enter

### Results
Each result shows:
- Item title (linked to the item)
- Nearest heading above the match (if any), e.g. "Project Background › Installation"
- A short text snippet (~150 chars) with the matching term(s) **highlighted** (bold or `<mark>`)
- Results ranked by: exact title match > heading match > body match; within each tier, rank
  by recency of last edit

### Backend
The search must be implemented server-side (full-text scanning of all item bodies in the
browser is impractical at scale).

#### Option A — Backend full-text search (preferred)
- Add a new API endpoint: `GET /api/search?q=TEXT&mode=fulltext`
- Returns `Array<{ itemId, itemTitle, headingPath, snippet, highlightRanges }>`
- Backend implementation options (choose based on existing DB):
  - **SQLite**: use `FTS5` virtual tables (`CREATE VIRTUAL TABLE items_fts USING fts5(body)`)
  - **Postgres**: use `tsvector` + `to_tsquery` with `ts_headline` for snippet extraction
- The index must be updated whenever an item's content changes (on save)

#### Option B — Client-side for small instances (fallback)
- Fetch all item summaries (titles + first 500 chars) via a dedicated endpoint and search in
  memory — acceptable only for small personal wikis (< 500 items)

Implement Option A; note Option B as a fallback if the backend work is out of scope.

### Migration / schema
- SQLite: new migration adding the FTS5 table and triggers to keep it in sync
- Postgres: new migration adding `tsvector` column and index; trigger to update on insert/update

### API client
- Add `searchFullText(query: string): Promise<SearchResult[]>` to `packages/api/src/`

## Implementation Notes
- Snippet extraction with highlighted ranges: the backend should return byte/char offsets so the
  frontend can insert `<mark>` tags without re-running the search
- `ts_headline` in Postgres does this automatically; SQLite FTS5 has `snippet()` and `highlight()`
  auxiliary functions
- ZealotScript body stored in the DB is the raw text format; for snippet purposes, strip
  `:::`, `**`, `[[…]]` with a lightweight regex pass (do not parse fully) before highlighting
- The search sidebar component should debounce the API call, show a loading indicator, and
  handle empty / error states

## Dependencies
- TASK-007 (search sidebar UI)
- TASK-017 (ZealotScript parser — for stripping markup from snippets)
- Backend DB migration infra

## Files Likely Involved
- `crates/zealot-api/src/http/` — new `search.rs` handler (or extend existing search handler)
- `crates/zealot-infra/migrations/sqlite/` — FTS5 migration
- `crates/zealot-infra/migrations/postgres/` — tsvector migration
- `crates/zealot-infra/src/repos/` — search repo implementation
- `packages/api/src/` — `searchFullText` API client method
- `packages/ui/src/views/` — update search sidebar to support full-text mode and result snippets
- `packages/content/src/css/` — snippet `<mark>` highlight styling
