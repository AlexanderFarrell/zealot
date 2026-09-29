# Epic 13 — Gaps from Documentation

This epic was created by reading all of Zealot's documentation end-to-end and identifying
real pain points, inconsistencies between docs and code, missing features, and improvements
that would make Zealot more robust, easier to deploy, and easier to use.

---

## Ticket Index

### Small cleanups (this epic)

| Ticket | Title | Type |
|---|---|---|
| [GAP-001](GAP-001-cors-env-var.md) | CORS origin configurable via env var | Improvement |
| [GAP-002](GAP-002-account-creation-env.md) | Wire `ACCOUNT_CREATION_ENABLED` into server binary | Bug |
| [GAP-003](GAP-003-admonition-syntax-in-rules-docs.md) | Fix admonition syntax in rules-engine cookbook | Doc fix |
| [GAP-004](GAP-004-os-date-in-overview-examples.md) | Fix `os.date` in overview Lua examples | Doc/code fix |
| [GAP-005](GAP-005-lua-item-id-type.md) | Clarify item ID type in Lua rules doc | Doc fix |
| [GAP-006](GAP-006-planner-curl-example.md) | Fix curl example using `?date=` query param | Doc fix |
| [GAP-007](GAP-007-git-clone-url.md) | Fix placeholder git clone URL in quickstart | Doc fix |
| [GAP-008](GAP-008-zealot-compose-port.md) | Port mismatch in `zealot-compose.yml` (8080 vs 8085) | Bug/Doc |
| [GAP-009](GAP-009-nginx-upstream-env.md) | Make nginx upstream configurable (not hardcoded) | Improvement |
| [GAP-010](GAP-010-mysql-removal.md) | Remove stale MySQL references from docs | Doc fix |
| [GAP-011](GAP-011-screenshot-placeholders.md) | Replace screenshot placeholders in overview.md | Chore |
| [GAP-012](GAP-012-password-reset.md) | Add password reset endpoint | Feature |
| [GAP-013](GAP-013-mcp-filter-tool.md) | Add `filter_items` tool to MCP server | Feature |

### Data integrity / dangerous operations

| Ticket | Title | Type |
|---|---|---|
| [GAP-014](GAP-014-attr-kind-delete-guard.md) | Guard against deleting attribute kinds with existing data | Safety |
| [GAP-015](GAP-015-item-type-delete-guard.md) | Guard against deleting item types with assigned items | Safety |
| [GAP-016](GAP-016-orphaned-children-on-delete.md) | Orphaned children when a parent item is deleted | Bug |

### Lua rules engine gaps

| Ticket | Title | Type |
|---|---|---|
| [GAP-017](GAP-017-lua-add-link.md) | Lua rules cannot create or remove item links | Feature |
| [GAP-018](GAP-018-lua-get-children-related.md) | Lua rules cannot read children or related items | Feature/Bug |
| [GAP-019](GAP-019-lua-planner-access.md) | Lua rules cannot access planner data | Feature |
| [GAP-020](GAP-020-lua-repeats-access.md) | Lua rules cannot read or update repeat/habit entries | Feature |
| [GAP-021](GAP-021-lua-comment-read.md) | Lua rules cannot read comments (add-only) | Feature |

### MCP tool gaps

| Ticket | Title | Type |
|---|---|---|
| [GAP-022](GAP-022-mcp-create-item-links.md) | MCP `create_item` cannot set links | Feature |
| [GAP-023](GAP-023-mcp-annual-planner.md) | MCP missing annual planner tool | Feature |
| [GAP-024](GAP-024-mcp-repeat-range.md) | MCP repeat entries single-day only (no range) | Feature |
| [GAP-025](GAP-025-mcp-media-upload.md) | MCP cannot upload or rename files | Feature |
| [GAP-026](GAP-026-mcp-trigger-descriptions.md) | MCP automation tool lists only 4 of 10 trigger kinds | Doc/Bug |
| [GAP-027](GAP-027-mcp-type-attr-association.md) | MCP cannot manage type–attribute associations | Feature |

### Search and scale

| Ticket | Title | Type |
|---|---|---|
| [GAP-028](GAP-028-search-pagination.md) | Search hard-capped at 20 results, no pagination | Bug |
| [GAP-029](GAP-029-filter-pagination.md) | `POST /item/filter` returns unbounded result sets | Improvement |

### Wiki link fragility

| Ticket | Title | Type |
|---|---|---|
| [GAP-030](GAP-030-wiki-links-not-auto-updated.md) | Wiki links not updated on save (manual rebuild only) | Bug |
| [GAP-031](GAP-031-title-rename-breaks-links.md) | Renaming an item title silently breaks inbound wiki links | Bug |

### Repeat / habit system

| Ticket | Title | Type |
|---|---|---|
| [GAP-032](GAP-032-repeat-items-list.md) | No endpoint to list all items enrolled in the repeat tracker | Feature |
| [GAP-033](GAP-033-repeat-range-query.md) | No date-range query for repeat entries | Feature |

### Planner design

| Ticket | Title | Type |
|---|---|---|
| [GAP-034](GAP-034-planner-hardcoded-attr-keys.md) | Planner attribute keys are hardcoded magic strings | Bug/Improvement |
| [GAP-035](GAP-035-daily-planner-week-items.md) | Weekly items don't appear in daily planner view | Bug |

### Comment system

| Ticket | Title | Type |
|---|---|---|
| [GAP-036](GAP-036-comment-server-timestamp.md) | Comments require client-provided timestamp (no server default) | Improvement |
| [GAP-037](GAP-037-comment-search-range.md) | No comment search or date-range query | Feature |

### Session / auth hygiene

| Ticket | Title | Type |
|---|---|---|
| [GAP-038](GAP-038-session-gc.md) | Expired sessions never garbage-collected from the database | Improvement |

### Larger improvements (reference to other epics)

| Ticket | Title | Belongs in |
|---|---|---|
| [LARGER-001](LARGER-001-cors-configurable.md) | Pre-built image release pipeline | New deployment epic |
| [LARGER-002](LARGER-002-backup-built-in.md) | Built-in scheduled backup mechanism | Ops/Infrastructure epic |
