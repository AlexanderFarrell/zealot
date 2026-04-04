# Plan: Implement Comment API, Service, and SQLite Repo

## Context

The comment feature has a partial scaffold: domain types exist, the repo trait and SQLite stub are in place, and the service has method signatures — but everything is `todo!()`. This plan fully implements comments end-to-end: SQLite repo → service → HTTP API.

The `comment` table has no `account_id` column; authorization is enforced by joining `item` (which has `account_id`). The `Comment` domain struct holds a full `Item`, but the repo only knows `item_id`. We resolve this by introducing `CommentCore` (mirrors `ItemCore`) and having `CommentService` hydrate via `ItemService`.

## Files to Modify

| File | Change |
|---|---|
| `crates/zealot-domain/src/comment.rs` | Add `CommentCore` struct |
| `crates/zealot-domain/src/lib.rs` | No change needed (comment already exported) |
| `crates/zealot-app/src/repos/comment.rs` | Change trait to use `CommentCore` |
| `crates/zealot-app/src/services/comment.rs` | Implement all methods; take `Arc<ItemService>` |
| `crates/zealot-app/src/services/mod.rs` | Wire `CommentService` with its dependencies |
| `crates/zealot-infra/src/repos/sqlite/comment_sqlite.rs` | Implement all repo methods |
| `crates/zealot-api/src/http/comment.rs` | Create HTTP handlers and routes |
| `crates/zealot-api/src/http/mod.rs` | Register `comment` module and nest `/comment` routes |

## Step 1 — Domain: Add `CommentCore`

In `crates/zealot-domain/src/comment.rs`, add:

```rust
#[derive(Debug, Clone)]
pub struct CommentCore {
    pub comment_id: Id,
    pub item_id: Id,
    pub timestamp: NaiveDateTime,
    pub content: String,
}
```

`Comment` (with full `Item`) stays for the service/API layer. `CommentCore` is the repo's return/input type.

## Step 2 — Repo Trait: Use `CommentCore`

In `crates/zealot-app/src/repos/comment.rs`, change the trait so all methods work with `CommentCore` instead of `Comment`:

```rust
pub trait CommentRepo: Debug + Send + Sync {
    fn get_for_day(&self, day: &NaiveDate, account_id: &Id) -> Result<Vec<CommentCore>, RepoError>;
    fn get_for_item(&self, item_id: &Id, account_id: &Id) -> Result<Vec<CommentCore>, RepoError>;
    fn add_comment(&self, dto: &AddCommentDto, account_id: &Id) -> Result<Option<CommentCore>, RepoError>;
    fn update_comment(&self, dto: &UpdateCommentDto, account_id: &Id) -> Result<Option<CommentCore>, RepoError>;
    fn delete_comment(&self, comment_id: &Id, account_id: &Id) -> Result<(), RepoError>;
}
```

## Step 3 — SQLite Repo: Implement All Methods

In `crates/zealot-infra/src/repos/sqlite/comment_sqlite.rs`, implement using the `block_in_place` + `block_on` async pattern (same as `ItemSqliteRepo`).

**Row struct:**
```rust
#[derive(sqlx::FromRow)]
struct CommentRow {
    comment_id: i64,
    item_id: i64,
    time: i64,      // stored as unix timestamp (integer)
    content: String,
}
```

**Helper** to convert row → `CommentCore`:
```rust
fn row_to_comment_core(row: CommentRow) -> Result<CommentCore, RepoError> { ... }
// NaiveDateTime::from_timestamp_opt(row.time, 0)
```

**`get_for_day`** — join `comment` with `item` to enforce account ownership, filter by unix timestamp range for the day:
```sql
SELECT c.comment_id, c.item_id, c.time, c.content
FROM comment c
JOIN item i ON i.item_id = c.item_id
WHERE i.account_id = ?
  AND c.time >= ? AND c.time < ?
ORDER BY c.time ASC
```

**`get_for_item`** — join to verify ownership:
```sql
SELECT c.comment_id, c.item_id, c.time, c.content
FROM comment c
JOIN item i ON i.item_id = c.item_id
WHERE c.item_id = ? AND i.account_id = ?
ORDER BY c.time ASC
```

**`add_comment`** — parse timestamp string, insert, RETURNING:
```sql
INSERT INTO comment (item_id, time, content) VALUES (?, ?, ?)
RETURNING comment_id, item_id, time, content
```
Verify the item belongs to the account before inserting (select item WHERE item_id=? AND account_id=?; return RepoError::NotFound if absent).

**`update_comment`** — UPDATE with COALESCE, join to check ownership, then SELECT:
```sql
UPDATE comment SET
  time = COALESCE(?, time),
  content = COALESCE(?, content),
  last_updated = strftime('%s','now')
WHERE comment_id = ?
  AND item_id IN (SELECT item_id FROM item WHERE account_id = ?)
```

**`delete_comment`** — DELETE enforcing ownership via subquery:
```sql
DELETE FROM comment
WHERE comment_id = ?
  AND item_id IN (SELECT item_id FROM item WHERE account_id = ?)
```

**Timestamp parsing helper:** the `AddCommentDto.timestamp` and `UpdateCommentDto.timestamp` are strings in `"%Y-%m-%d %H:%M:%S"` format (matching `CommentDto` output). Parse with `NaiveDateTime::parse_from_str`, map parse errors to `RepoError::DatabaseError`.

## Step 4 — Service: Implement Methods + Hydration

In `crates/zealot-app/src/services/comment.rs`:

- Add `item_service: Arc<ItemService>` field
- Update `new()` to accept it
- Expand `CommentServiceError`:
  ```rust
  pub enum CommentServiceError {
      NotFound,
      Unauthorized,
      Repo(#[from] RepoError),
  }
  ```
- Add private `hydrate` method converting `CommentCore` → `Comment` via `item_service.get_item_by_id()`; return `CommentServiceError::NotFound` if item not found
- Implement all 5 public methods by calling the repo, then hydrating results

## Step 5 — Wire `CommentService` in `ZealotServices`

In `crates/zealot-app/src/services/mod.rs`:

- Add `pub comment: Arc<CommentService>` to `ZealotServices`
- In `ZealotServices::new()`, construct it **after** `item` (it depends on `item`):
  ```rust
  let item = Arc::new(ItemService::new(...));
  let comment = Arc::new(CommentService::new(&repos.comment, &item));
  Self { comment, item, ... }
  ```

## Step 6 — HTTP API

Create `crates/zealot-api/src/http/comment.rs` following the `item.rs` pattern exactly:

**Routes:**
```
GET  /comment/day/:date      → get_for_day      (date: "YYYY-MM-DD" path param)
GET  /comment/item/:item_id  → get_for_item
POST /comment/               → add_comment      (body: AddCommentDto)
PATCH /comment/:comment_id   → update_comment   (body: UpdateCommentDto)
DELETE /comment/:comment_id  → delete_comment
```

**Error mapping:**
```rust
fn comment_service_err(err: CommentServiceError) -> HttpError {
    match err {
        CommentServiceError::NotFound => HttpError::NotFound,
        CommentServiceError::Unauthorized => HttpError::Unauthorized,
        CommentServiceError::Repo(_) => HttpError::Internal,
    }
}
```

All handlers: extract `Extension(actor)`, call `require_account(&actor)?`, call `state.services.comment.*`, map to `CommentDto`.

## Step 7 — Register Routes

In `crates/zealot-api/src/http/mod.rs`:
- Add `mod comment;`
- Add `.nest("/comment", comment::routes(state.clone()))` to `build_router`

## Verification

1. `cargo build -p zealot-infra` — confirms repo compiles
2. `cargo build` — full build across all crates
3. `cargo test` — run existing test suite
4. Manual curl smoke test (requires a running server with a valid session cookie or API key):
   - `POST /comment/` with `{"item_id": 1, "timestamp": "2026-04-04 10:00:00", "content": "test"}`
   - `GET /comment/item/1`
   - `GET /comment/day/2026-04-04`
   - `PATCH /comment/1` with partial update
   - `DELETE /comment/1`
