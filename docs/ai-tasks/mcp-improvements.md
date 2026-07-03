# Zealot MCP v2 — Agent-First Redesign

## Status

Completed on 2026-07-03. Final verification:

- `cargo check -p zealot-mcp`
- `cargo test -p zealot-mcp`

## Context

The MCP server (`apps/mcp/`) predates the CLI/TUI work and has fallen behind. Problems:

- **Context bloat**: every read tool returns full item bodies as *pretty-printed* JSON. `list_items`, `get_children`, `get_related_items`, and planner views are unbounded — one call can dump the whole wiki into an agent's context.
- **Bypasses the typed client**: tools hand-build URL path strings against raw `client.get/post/...` instead of using `crates/zealot-client/src/api/*.rs`, duplicating every endpoint (drift risk; prompts even use wrong trailing-slash paths).
- **Missing capability vs the CLI**: no backlinks, most-viewed/random, media get/upload/rename, append, quick journal, day dashboard, planner year, attribute rename, id-or-title item refs, create-with-types/links/parent.
- **Inconsistencies**: `id` vs `item_id` param names, duplicated `pretty()` helper ×5, coarse error mapping, prompts silently swallow errors.

**User decisions**: redesign freely (breaking changes OK — "MCP v2"); multi-item tools return **summaries by default** with a `detail` opt-in; scope = agent-first composites + gap-fill + analysis tools. Out of scope: MCP resources, PDF/DOCX export, account/API-key tools, backend changes.

## Design decisions

- **Output**: compact JSON (`serde_json::to_string`), not pretty — structured data must stay machine-quotable for write calls, and compact saves ~30% tokens. List tools return an envelope `{"count":N,"next_offset":123,"items":[...]}` (`next_offset` present iff `count == limit`).
- **Detail levels**: `detail` enum param — `meta` (id/title/types), `summary` (**default** for multi-item tools: + attrs + 120-char whitespace-collapsed preview + `content_chars`), `full` (complete `ItemDto`). `get_item` defaults to `full`.
- **Unified item refs**: every item-identifying param is `item: String` accepting numeric ID (`"42"`/`"#42"`) or exact title — mirrors CLI `ItemRef` (`apps/cli/src/args.rs`) / `Ctx::resolve_item` (`apps/cli/src/context.rs:55`).
- **All tools move onto the typed client** — verified every needed method exists in `crates/zealot-client/src/api/{items,planner,repeats,comments,time_blocks,media,item_types,attributes,rules}.rs`; **no client additions needed**. Planner/repeat methods take `chrono::NaiveDate` → parse date strings with proper `invalid_params` errors.
- **Final tool count: 53** (was 51) — consolidation pays for the new capability.

## New/changed files

```
apps/mcp/src/
  output.rs        NEW — Detail enum, ItemSummary::project(&ItemDto, Detail), preview(),
                         Page<T> envelope, item_page(), json_result() (compact JSON)
  refs.rs          NEW — ItemRef::parse ("#42"|"42"→Id, else Title) +
                         ZealotServer::resolve_item(&str) -> Result<ItemDto, McpError>
  config.rs        + journal_item: Option<String> (env ZEALOT_JOURNAL_ITEM)
  tools/mod.rs     ZealotServer gains journal_item field; api_err → err_ctx(resource, e):
                   NotFound & 4xx → invalid_params with resource name; 5xx → internal_error.
                   Merge new analysis_tool_router. Rewrite get_info() instructions.
  tools/wiki.rs        rewrite → 14 tools
  tools/planner.rs     rewrite → 10 tools
  tools/time_block.rs  rewrite → 4 tools
  tools/automation.rs  refactor → 15 tools (names mostly kept)
  tools/media.rs       expand → 6 tools
  tools/analysis.rs    NEW → 4 tools
  prompts/mod.rs   light touch (typed client, propagate errors, summary projections)
apps/mcp/tests/tools_tests.rs  rewrite alongside (wiremock harness kept)
apps/mcp/Cargo.toml  drop urlencoding; version bump; add chrono if not present
```

## v2 tool surface (53)

Conventions: verb-first snake_case; `item` = id-or-title ref; dates `YYYY-MM-DD`; multi-item tools take `detail`/`limit`/`offset`.

### wiki (14)
| Tool | Notes |
|---|---|
| `get_item(item, detail=full)` | merges get_item + get_item_by_title |
| `get_item_outline(item)` | NEW — headings (`^#{1,6} `) → `[{level,text}]` + content_chars + attr keys; pure MCP-side parse |
| `browse_items(mode: root\|recent\|random\|most_viewed, type_filter?, limit, offset, detail)` | merges list_items/list_recent_items + adds random/most_viewed (`most_viewed` rows are `{id,title,view_count}` — note in description); root/type sliced MCP-side |
| `search_items(term, scope?, regex?, limit=20, offset, detail)` | summary rows keep `snippet`+`match_scope` |
| `filter_items(filters[], limit=50, offset, detail)` | typed `AttributeFilterDto` |
| `get_linked_items(item, direction: children\|related\|backlinks, detail, limit, offset)` | merges children/related + NEW backlinks |
| `create_item(title, content?, attributes?, types?, links?, parent?)` | full `AddItemDto`; `parent` ref resolved → `attributes["Parent"]=id` (CLI pattern, `apps/cli/src/commands/item.rs:147`) |
| `update_item(item, title?, content?)` | |
| `append_to_item(item, text)` | NEW — resolve, newline-join (CLI `append`), update; returns `{id,title,content_chars}` |
| `delete_item(item)` | |
| `update_item_attributes(item, set?, remove?, rename?)` | merges set/delete + NEW rename; order set→rename→remove |
| `assign_item_type(item, type_name)` / `unassign_item_type(...)` | now take item refs |
| `rebuild_links()` | NEW |

### planner (10)
| Tool | Notes |
|---|---|
| `day_dashboard(date?)` | NEW composite mirroring CLI `day` (`apps/cli/src/commands/planner.rs:14`): `tokio::join!(planner_day, repeats_for_day, time_blocks_for_day, comments_for_day)` → `{date, plan:[summary], habits:[{id,title,status,comment}], time_blocks:[{...,start:"HH:MM"}], journal:[...]}` |
| `get_plan(period, detail, limit, offset)` | period by shape: `YYYY-MM-DD`→day, `YYYY-Wnn`→week, `YYYY-MM`→month, `YYYY`→year (NEW) |
| `list_habits(detail)` | was get_repeat_items |
| `get_habit_entries(start_date, end_date?)` | merges day+range; compact rows `{item_id,title,date,status,comment}` — no embedded full items |
| `set_habit_status(item, date, status?, comment?)` | item ref |
| `get_comments(item? XOR date?)` | merges for_item/for_day; compact rows |
| `add_comment(item, content, timestamp?)` | timestamp defaults to now |
| `add_journal_entry(content)` | NEW — uses configured `journal_item`; helpful error when unset (CLI `journal` pattern, `apps/cli/src/commands/comment.rs:77`) |
| `update_comment(comment_id, content)` / `delete_comment(comment_id)` | |

### time_block (4)
`get_time_blocks(start_date, end_date?, item?)` (merges 3 read tools), `create_time_block(item, date, start:"HH:MM", end:"HH:MM", note?)` (HH:MM parsed to minutes — agents are bad at minutes-since-midnight), `update_time_block(block_id, ...)`, `delete_time_block(block_id)`.

### automation (15, refactor)
Same six rule tools + item-type CRUD + attribute-kind CRUD, all onto typed methods. `list_item_types` switches to `item_type_summaries()` (includes item_count — strictly better for agents). Renames: `get_item_type_by_name`→`get_item_type`, `get_attribute_by_key`→`get_attribute_kind`. `list_rules` projects out script bodies (`script_chars` instead); full script via `get_rule`. Keep the `JsonObject` schemars shim. Item-type↔attr-kind attach/detach stays unexposed (no typed method; out of scope).

### media (6)
`list_media(path?)`, **NEW** `get_media(path)` (`media_get`; image → `Content::image` base64; text ≤100 KB → text with truncation notice; other/oversize >2 MB → metadata JSON), **NEW** `upload_media(path, content, encoding: text|base64)`, `create_media_folder(folder)`, **NEW** `rename_media(old_location, new_name)`, `delete_media(path)`.

### analysis (4, NEW module)
Private helper `fetch_all_items(cap=1000)` — pages `recent_items(100, offset)` to exhaustion (≤10 calls), returns `(items, truncated)`; `truncated` surfaced in output.

| Tool | Computation |
|---|---|
| `wiki_stats()` | `join!(item_type_summaries, list_attribute_kinds, recent_items(10,0))` → totals, per-type counts, attr-kind count, recently modified (meta) — 3 calls |
| `orphaned_items(limit?)` | fetch_all_items; orphan = empty `links` AND no other item links to it (uses `links[].other_item_id` set — zero backlink calls) |
| `attribute_usage()` | fetch_all_items + list_attribute_kinds; usage count per key; flags unused kinds & undefined-but-used keys |
| `habit_stats(start_date, end_date)` | one `repeats_for_range`; per habit: counts, completion_rate, current/longest streak (Complete extends, Skip/Alternate neutral, Not Complete breaks); range capped 366 days |

## Prompts (light touch)
Typed client calls (fixes trailing-slash paths), errors propagate via `err_ctx` instead of `unwrap_or(empty)`, embedded JSON uses `ItemSummary` projections, `review_tasks` per-day loop (≤15 calls) → one `repeats_for_range`.

## Server instructions rewrite (`get_info()`)
Teach v2 idioms: item refs (id or exact title); **browse-then-read** (lists return summaries — use `detail:"full"` or `get_item` for content; `get_item_outline` for long items); `{count,next_offset,items}` paging; date/period/time formats; start-of-day → `day_dashboard`; habit statuses; ZealotScript content.

## Implementation phases (each ends `cargo check -p zealot-mcp && cargo test -p zealot-mcp`)

1. **Infra**: `output.rs`, `refs.rs`, `err_ctx` (keep `api_err` alias temporarily), config `journal_item`, `ZealotServer` field. Add a schema snapshot test early to verify rmcp emits lowercase string enums for `Detail`/`LinkDirection` (same risk class as the existing `JsonObject` shim).
2. **wiki.rs** rewrite + tests — establishes every pattern.
3. **planner.rs + time_block.rs** rewrite (incl. `day_dashboard`, `add_journal_entry`) + tests.
4. **media.rs + automation.rs** + tests.
5. **analysis.rs** + router merge; delete `api_err` alias, `urlencoding` dep, duplicated `pretty()` helpers.
6. **Prompts cleanup + instructions rewrite**; final test pass; version bump.

## Testing / verification

- **Unit** (`output.rs`): projection per detail level, preview char-safety/truncation, `next_offset` iff full page.
- **Wiremock integration** (`tests/tools_tests.rs`, existing harness): refs (id vs `#42` vs title → correct routes, url-encoded); one mock per consolidated-tool mode (`browse_items` ×4, `get_plan` ×4 period shapes, `get_time_blocks` item vs range); `day_dashboard` mounts 4 mocks each hit once; `append_to_item` PATCH body has newline-joined content; `habit_stats` streak math fixture; `orphaned_items` 2-page pagination + link graph; `err_ctx` (404→invalid_params w/ resource, 422→invalid_params, 500→internal_error); `get_media` directory vs image vs oversize; `review_tasks` issues exactly one `/repeat/range` call.
- **End-to-end**: run `zealot-mcp` in stdio mode against the live server and exercise `day_dashboard`, `browse_items` (confirm summary output + paging envelope), `get_item` by title, `append_to_item`, `habit_stats` via an MCP client (or the reconnected `zealot` MCP in Claude Code).
- Assertions parse compact JSON (`serde_json::from_str`), not string-match pretty output.
