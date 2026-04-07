# Plan: Audit & Fix Frontend API to Match Backend

## Context
The backend (Rust/Axum) defines HTTP endpoints in `crates/zealot-api/src/http/`. The frontend TypeScript API client lives in `packages/api/src/`. The goal is to make the frontend client match the backend exactly — correct paths, methods, request/response shapes, and add missing endpoints.

---

## Discrepancies Found

### 1. `comment.ts` — Wrong URL prefix
- **Frontend uses**: `/comments/...` (plural)
- **Backend expects**: `/comment/...` (singular)
- Fix all 4 methods: `GetForDay`, `GetForItem`, `UpdateEntry`, `DeleteEntry`
- Also: `AddComment` method is **missing** — backend has `POST /comment/`

### 2. `comment.ts` — Missing `AddComment`
- Backend: `POST /comment/` with `AddCommentDto` body → returns `CommentDto`
- Frontend: no such method exists
- Add `AddComment(dto: AddCommentDto): Promise<Comment>`

### 3. `attribute.ts` — Wrong DELETE URL
- **Frontend**: `DELETE /item/${item_id}/${key}` (missing `attr` segment)
- **Backend**: `DELETE /item/{item_id}/attr/{key}`
- Fix `remove()` to use `/item/${item_id}/attr/${key}`

### 4. `attribute.ts` — Wrong body for `set_value`
- **Frontend**: sends `{ key: value }` (uses key as field name)
- **Backend**: `PATCH /item/{item_id}/attr` expects `HashMap<String, serde_json::Value>` — i.e. `{ [key]: value }`
- Frontend already does `{ key: value }` which IS the right shape — but variable naming is confusing. Actually the body should be `{ [key]: value }` as a dynamic object. Current code sends `{ key: value }` where `key` is a literal string, not the variable. **This is a bug.**
- Fix: `await patch_req(..., { [key]: value })`

### 5. `attribute_kind.ts` — Wrong base URL
- **Frontend base URL**: `/item/kind`
- **Backend routes**: `/attribute/` (prefix is `/attribute`)
- `BasicAPI` uses `${URL}/id/${id}` for get_by_id — backend has `GET /attribute/id/{kind_id}` ✓
- But base `GET /item/kind` should be `GET /attribute/` ✗
- Fix constructor: `super(`${baseURL}/attribute`, dto_factory)`
- Also: `UpdateConfig` calls `PATCH ${URL}/${kind_id}/config` — **backend has no `/config` sub-route**. Backend uses `PATCH /attribute/id/{kind_id}` for updates. Remove `UpdateConfig` or fold into `update()`.

### 6. `item_type.ts` — Wrong base URL and wrong assign endpoints
- **Frontend base URL**: `/item/type`
- **Backend routes**: `/item_type/` (prefix is `/item_type`)
- Fix constructor: `super(`${baseURL}/item_type`, dto_factory)`
- `BasicAPI.get_by_id` uses `${URL}/id/${id}` — backend has `GET /item_type/{type_id}` (no `/id/` segment). **Mismatch** — but this is in `BasicAPI` itself; the backend for item_type doesn't have `/id/` prefix. May need to override `get_by_id`.
- **assign endpoint**: Frontend calls `POST /item/type/assign/${item_type_name}` with `{ attribute_kinds: [...] }` — Backend expects `POST /item_type/{type_id}/attr_kind` with `Vec<String>` (array, not object)
- **unassign endpoint**: Frontend calls `DELETE /item/type/assign/${item_type_name}` — Backend expects `DELETE /item_type/{type_id}/attr_kind` with `Vec<String>`
- The frontend uses `item_type_name` but backend uses `type_id` (i64). Need to align — use `type_id: number`.

### 7. `repeat.ts` — Wrong method and URL for `SetStatus`
- **Frontend**: `PATCH /repeat/${dto.item_id}/day/${iso}` with `{ status, comment }`
- **Backend**: `PUT /repeat/status` with `UpdateRepeatEntryDto` body, returns 204
- Fix: change to `put_json` (or `put_req`) to `PUT /repeat/status` passing the full dto

### 8. `index.ts` — `MediaAPI` and `ItemAPI` not wired up
- `MediaAPI` exists in `media.ts` but is not imported or added to `ZealotAPI`
- No `ItemAPI` exists at all — backend has ~15 item endpoints under `/item/`
- Add `MediaAPI` to `ZealotAPI`
- Create `item.ts` with `ItemAPI` class covering all item endpoints

### 9. Missing `ItemAPI` entirely
Backend has these item endpoints with no frontend equivalent:
- `GET /item/` (optional `?type=` query param)
- `GET /item/title/{title}`
- `GET /item/id/{item_id}`
- `GET /item/search?term=`
- `GET /item/children/{item_id}`
- `GET /item/related/{item_id}`
- `POST /item/filter` with `{ filters: AttributeFilterDto[] }`
- `POST /item/` with `AddItemDto`
- `PATCH /item/{item_id}` with `UpdateItemDto`
- `DELETE /item/{item_id}`
- `PATCH /item/{item_id}/attr` (handled in `AttributeAPI` ✓)
- `PATCH /item/{item_id}/attr/rename` (handled in `AttributeAPI` ✓)
- `DELETE /item/{item_id}/attr/{key}` (handled in `AttributeAPI`, fix needed ✓)
- `POST /item/{item_id}/assign_type/{type_name}`
- `DELETE /item/{item_id}/assign_type/{type_name}`

---

## Files to Modify

| File | Changes |
|------|---------|
| [packages/api/src/comment.ts](packages/api/src/comment.ts) | Fix `/comments/` → `/comment/`, add `AddComment` |
| [packages/api/src/attribute.ts](packages/api/src/attribute.ts) | Fix DELETE path (add `attr/`), fix `set_value` body (`{ [key]: value }`) |
| [packages/api/src/attribute_kind.ts](packages/api/src/attribute_kind.ts) | Fix base URL `/item/kind` → `/attribute`, remove/fix `UpdateConfig` |
| [packages/api/src/item_type.ts](packages/api/src/item_type.ts) | Fix base URL `/item/type` → `/item_type`, fix assign/unassign endpoints to use `type_id` and correct paths/body shape |
| [packages/api/src/repeat.ts](packages/api/src/repeat.ts) | Change PATCH to PUT, fix URL to `/repeat/status` |
| [packages/api/src/index.ts](packages/api/src/index.ts) | Import and expose `MediaAPI` and new `ItemAPI` |

## Files to Create

| File | Purpose |
|------|---------|
| [packages/api/src/item.ts](packages/api/src/item.ts) | New `ItemAPI` class with all item CRUD + search + filter + type assignment |

---

## Decisions Made

1. `item_type` assign/unassign: use `type_id: number` — match backend exactly
2. `AttributeKindAPI.UpdateConfig`: keep as-is (planned future endpoint)
3. `item_type.ts` `get_by_id`: override in `ItemTypeAPI` to use `/${id}` instead of `/id/${id}` since backend uses `GET /item_type/{type_id}` directly
4. `ItemAPI`: create `item.ts` with full item endpoints

---

## Verification
- Run frontend build: `cd packages/api && npm run build` (or equivalent)  
- Manually test each endpoint against a running backend
- Check TypeScript types align with domain DTOs in `packages/domain/src/`
