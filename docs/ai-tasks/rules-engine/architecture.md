# Rules Engine Architecture

## Crate structure

A new crate `crates/zealot-lua` is added to the workspace. It is the only place `mlua` is a dependency. Only `apps/server` links it; other crates (`zealot-infra`, `zealot-api`) stay clean.

```
crates/
  zealot-domain/      ← Rule, TriggerKind structs + DTOs
  zealot-app/         ← RuleRepo trait, EventPort, RuleRunnerPort, RuleService, scheduler
  zealot-lua/         ← LuaRuleRunner concrete impl (sandbox + bindings)  [NEW]
  zealot-infra/       ← RuleSqliteRepo, RulePostgresRepo, BroadcastEventPort
  zealot-api/         ← HTTP routes for /rule
apps/
  server/             ← wires LuaRuleRunner into ports, starts scheduler + event listener
```

---

## Domain model (`crates/zealot-domain/src/rule.rs`)

```rust
pub struct Rule {
    pub rule_id:     Id,
    pub account_id:  Id,
    pub name:        String,
    pub description: String,
    pub trigger:     TriggerKind,
    pub script:      String,           // Lua source
    pub enabled:     bool,
    pub created_at:  NaiveDateTime,
    pub last_run_at: Option<NaiveDateTime>,
    pub last_error:  Option<String>,
    pub last_output: Option<String>,   // captured from zealot.notify() calls
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TriggerKind {
    // Event-based
    OnItemCreate,
    OnItemUpdate,
    OnItemDelete,
    OnCommentAdd,
    OnTypeAssign   { type_name: Option<String> },
    OnTypeUnassign { type_name: Option<String> },
    OnAttributeSet { attribute_key: Option<String> },
    // Scheduled
    Cron     { expression: String },   // 5-field Unix cron
    Interval { seconds: u64 },
    // Manual
    Manual,
}
```

**DTOs** follow the exact pattern of `ItemDto`:
- `RuleDto` (send to client) — all fields, `trigger` serialized as tagged JSON
- `AddRuleDto` (receive from client) — name, description, trigger, script
- `UpdateRuleDto` (receive from client) — all optional fields + rule_id

---

## Event system (`crates/zealot-app/src/ports/events.rs`)

```rust
pub enum ZealotEvent {
    ItemCreated   { account_id: Id, item: ItemDto },
    ItemUpdated   { account_id: Id, item: ItemDto },
    ItemDeleted   { account_id: Id, item_id: Id },
    CommentAdded  { account_id: Id, comment: CommentDto },
    TypeAssigned  { account_id: Id, item: ItemDto, type_name: String },
    TypeUnassigned{ account_id: Id, item: ItemDto, type_name: String },
    AttributeSet  { account_id: Id, item: ItemDto, key: String },
}

pub trait EventPort: Debug + Send + Sync {
    fn emit(&self, event: ZealotEvent);   // fire-and-forget
}
```

`ZealotPorts` gains `events: Arc<dyn EventPort>`. The concrete implementation in `zealot-infra` wraps a `tokio::sync::broadcast::Sender<ZealotEvent>`.

Each `ItemService` mutation (add, update, delete, assign_type, set_attribute) calls `self.event_port.emit(...)` after a successful DB write. Same for `CommentService::add_comment`.

### Anti-recursion guard

A rule running `zealot.items.update(...)` would re-emit `ItemUpdated`, potentially triggering the same rule again. Guard: the `LuaRuleRunner` sets a thread-local `RULE_DEPTH` counter before execution. The `EventPort::emit` impl skips broadcasting if `RULE_DEPTH > 0` on the calling thread.

---

## Rule runner port (`crates/zealot-app/src/ports/rule_runner.rs`)

```rust
pub trait RuleRunnerPort: Debug + Send + Sync {
    async fn run_event_rules(&self, event: ZealotEvent) -> Vec<RuleRunResult>;
    async fn run_rule(&self, rule: &Rule, context: RuleContext) -> RuleRunResult;
}

pub struct RuleRunResult {
    pub rule_id:     Id,
    pub success:     bool,
    pub output:      Option<String>,
    pub error:       Option<String>,
    pub duration_ms: u64,
}

pub enum RuleContext {
    Event(ZealotEvent),
    Scheduled { now: NaiveDateTime },
    Manual,
}
```

---

## Lua sandbox (`crates/zealot-lua/src/sandbox.rs`)

Per-execution steps:

1. `mlua::Lua::new()` — fresh VM per run (no state shared between executions)
2. Strip dangerous globals from `_G`: `io`, `os`, `require`, `dofile`, `loadfile`, `loadstring`, `package`, `debug`
3. Inject instruction-count hook: interrupt after **10 million** Lua VM instructions
4. Set wall-clock timeout: `tokio::time::timeout(Duration::from_secs(10), ...)`
5. Inject `zealot` global table (see below)
6. `lua.load(&rule.script).exec_async().await`
7. Capture `Ok` / `Err(LuaError)` into `RuleRunResult`
8. Write `last_run_at`, `last_error`, `last_output` back via `RuleRepo::record_run`

The instruction-count hook catches infinite loops. The wall-clock timeout is a backstop for scripts that yield repeatedly into async.

**Allowed globals:** `string`, `table`, `math`, `pairs`, `ipairs`, `next`, `select`, `tostring`, `tonumber`, `type`, `pcall`, `xpcall`, `error`, `assert`, `unpack`, `rawget`, `rawset`, `rawequal`, `ipairs`, `setmetatable`, `getmetatable`

---

## Lua API surface (`zealot.*`)

All functions are `lua.create_async_function(...)` bindings. The `account_id` is captured from the rule context closure — scripts cannot change or observe it.

### `zealot.items`

| Function | Signature | Description |
|---|---|---|
| `get` | `(id: string) → item\|nil` | Fetch item by ID |
| `get_by_title` | `(title: string) → item\|nil` | Fetch item by exact title |
| `find_by_type` | `(type_name: string) → item[]` | All items with given type |
| `search` | `(term: string) → item[]` | Full-text search by title |
| `filter` | `(filters: table[]) → item[]` | Attribute filter — each filter is `{key, op, value}` |
| `create` | `(title: string, opts?) → item` | Create new item; opts: `content`, `types[]`, `attributes{}` |
| `update` | `(id: string, opts) → item` | Update item; opts: `title`, `content` |
| `set_attribute` | `(id: string, key: string, value: any) → bool` | Set a single attribute |
| `assign_type` | `(id: string, type_name: string) → bool` | Assign type to item |
| `delete` | `(id: string) → bool` | Delete item |

**Item table fields** exposed to Lua:
```
id          string
title       string
content     string
attributes  table (key → value)
types       string[]  (type names)
links       {id: string, relationship: string}[]
```

### `zealot.comments`

| Function | Signature | Description |
|---|---|---|
| `add` | `(item_id: string, content: string) → comment` | Add comment to item |

### Utilities

| Global | Description |
|---|---|
| `zealot.notify(msg)` | Append string to `last_output` (visible in Rules screen after run) |
| `zealot.log(msg)` | Alias for `zealot.notify` |
| `zealot.event` | Read-only table with current event context (nil if not event-triggered) |
| `zealot.now` | ISO-8601 datetime string of execution start |
| `zealot.date` | Today's date string `YYYY-MM-DD` |

**`zealot.event` fields** (event-triggered only):

| Event kind | Fields available |
|---|---|
| `on_item_create` / `on_item_update` | `kind`, `item` (item table) |
| `on_item_delete` | `kind`, `item_id` |
| `on_comment_add` | `kind`, `item_id`, `content`, `timestamp` |
| `on_type_assign` / `on_type_unassign` | `kind`, `item`, `type_name` |
| `on_attribute_set` | `kind`, `item`, `attribute_key` |

---

## Scheduler (`crates/zealot-app/src/scheduler.rs`)

`pub fn start(state: AppState)` — called from `apps/server/src/main.rs` after `AppState::new`. Spawns a Tokio background task:

```
loop {
    sleep(60 seconds)
    rules = repo.get_enabled_scheduled_rules()   // returns (Rule, account_id) pairs
    for (rule, account_id) in rules:
        if is_due(rule, now):
            tokio::spawn(runner.run_rule(rule, Scheduled { now }))
}
```

Due-check logic:
- **Cron**: parse expression with `croner` crate, check if `now` falls in the minute window since `last_run_at`
- **Interval**: `now - last_run_at >= interval_seconds`

The `croner` crate (pure Rust) is added to workspace dependencies.

---

## HTTP API (`crates/zealot-api/src/http/rule.rs`)

```
GET    /rule               → list all rules for current account
POST   /rule               → create rule  (body: AddRuleDto)
GET    /rule/{id}          → get rule by id
PATCH  /rule/{id}          → update rule  (body: UpdateRuleDto)
DELETE /rule/{id}          → delete rule
POST   /rule/{id}/run      → run rule immediately, return RuleRunResult
```

All behind `auth_middleware`. Registered via `.nest("/rule", rule::routes(state))` in `http/mod.rs`.

---

## Database schema

Migration: `crates/zealot-infra/migrations/sqlite/0004_rules.sql`

```sql
create table rule (
    rule_id      text primary key,
    account_id   text not null references account(account_id) on delete cascade,
    name         text not null,
    description  text not null default '',
    trigger_kind text not null,
    trigger_config text not null default '{}',
    script       text not null default '',
    enabled      integer not null default 1,
    created_at   integer not null,
    last_run_at  integer,
    last_error   text,
    last_output  text
);
create index idx_rule_account     on rule(account_id);
create index idx_rule_event_rules on rule(account_id, enabled, trigger_kind);
```

`trigger_kind` is a string tag matching the `TriggerKind` serde rename (e.g. `"on_item_create"`, `"cron"`, `"manual"`). `trigger_config` stores the associated JSON payload (e.g. `{"expression":"0 9 * * 1"}`).

---

## Wire-up in `apps/server/src/main.rs`

```rust
let (event_tx, event_rx) = tokio::sync::broadcast::channel(1024);
let event_port = Arc::new(BroadcastEventPort::new(event_tx));
let rule_runner = Arc::new(LuaRuleRunner::new(repos.clone()));
let ports = ZealotPorts { events: event_port, rule_runner: rule_runner.clone(), ... };
let state = AppState::new(repos.clone(), ports);

zealot_app::scheduler::start(state.clone());

// Event listener task
tokio::spawn(async move {
    let mut rx = event_rx;
    loop {
        match rx.recv().await {
            Ok(event) => { tokio::spawn(rule_runner.run_event_rules(event)); }
            Err(_) => break,
        }
    }
});
```
