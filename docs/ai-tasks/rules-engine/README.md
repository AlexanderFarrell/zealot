# Zealot Rules Engine — Overview

## What is it?

The rules engine embeds **Lua 5.4** into the Zealot backend. Users write Lua scripts that run automatically in response to events (e.g. an item being created) or on a schedule (e.g. every day at 9 AM), or on demand. Scripts interact with Zealot data through a safe, sandboxed API surface.

## Why Lua?

- Tiny runtime (~300 KB), embeds trivially into Rust via `mlua`
- Well-understood scripting language — familiar to many users
- Proven sandboxing model (strip `io`, `os`, `require`; add instruction-count hook)
- Fast enough for per-event execution without a separate process
- Same language could power future ZealotScript template commands and custom report views

## Files in this folder

| File | Contents |
|---|---|
| `README.md` | This overview |
| `architecture.md` | Full system design: domain model, event system, Lua API, scheduler, sandbox |
| `use-cases.md` | High-value automation ideas with example Lua scripts |
| `zealotscript-integration.md` | How Lua-generated content renders in the ZealotScript editor |

## Ticket index

Backend tickets (prefix `TASK-LUA-`):

| Ticket | Description |
|---|---|
| [TASK-LUA-001](../rewrite/TASK-LUA-001-crate-setup.md) | Create `crates/zealot-lua`, add `mlua` to workspace |
| [TASK-LUA-002](../rewrite/TASK-LUA-002-domain-model.md) | Fill in `Rule` domain model and DTOs |
| [TASK-LUA-003](../rewrite/TASK-LUA-003-event-system.md) | Event system: `ZealotEvent` enum, `EventPort` trait, broadcast impl |
| [TASK-LUA-004](../rewrite/TASK-LUA-004-rule-runner-port.md) | `RuleRunnerPort` trait and result types |
| [TASK-LUA-005](../rewrite/TASK-LUA-005-repo-and-migration.md) | `RuleRepo` trait + SQLite/Postgres impls + DB migration |
| [TASK-LUA-006](../rewrite/TASK-LUA-006-rule-service.md) | `RuleService` CRUD and `run_rule_now` |
| [TASK-LUA-007](../rewrite/TASK-LUA-007-lua-sandbox.md) | Lua VM sandbox: strip globals, instruction hook, timeout |
| [TASK-LUA-008](../rewrite/TASK-LUA-008-lua-bindings.md) | `zealot.*` Lua API bindings (items, comments, notify) |
| [TASK-LUA-009](../rewrite/TASK-LUA-009-rule-runner-impl.md) | `LuaRuleRunner` implementation |
| [TASK-LUA-010](../rewrite/TASK-LUA-010-event-emission.md) | Emit events from service mutations (item, comment) |
| [TASK-LUA-011](../rewrite/TASK-LUA-011-scheduler.md) | Scheduler loop for cron/interval rules |
| [TASK-LUA-012](../rewrite/TASK-LUA-012-http-endpoints.md) | HTTP API routes for rules CRUD + manual run |

Frontend tickets:

| Ticket | Description |
|---|---|
| [TASK-LUA-FE-001](../rewrite/TASK-LUA-FE-001-domain-types.md) | `Rule` TypeScript domain type and DTOs |
| [TASK-LUA-FE-002](../rewrite/TASK-LUA-FE-002-api-client.md) | `RuleAPI` client implementation |
| [TASK-LUA-FE-003](../rewrite/TASK-LUA-FE-003-rules-screen.md) | Rules screen UI (list + edit views) |
| [TASK-LUA-FE-004](../rewrite/TASK-LUA-FE-004-navigation.md) | Add Rules to sidebar navigation |

## Dependency order

```
TASK-LUA-001 (crate)
    └── TASK-LUA-002 (domain)
        ├── TASK-LUA-003 (events)
        ├── TASK-LUA-004 (runner port)
        └── TASK-LUA-005 (repo + migration)
            └── TASK-LUA-006 (service)
                ├── TASK-LUA-007 (sandbox)  ─┐
                └── TASK-LUA-008 (bindings) ─┴─ TASK-LUA-009 (runner impl)
                                                      └── TASK-LUA-010 (event emission)
                                                      └── TASK-LUA-011 (scheduler)
                                                      └── TASK-LUA-012 (HTTP)

Frontend (parallel, depends on TASK-LUA-012 being done for real API):
    TASK-LUA-FE-001 → TASK-LUA-FE-002 → TASK-LUA-FE-003 → TASK-LUA-FE-004
```
