# Desktop Epics Boundary: Epic 5 vs 8 vs 9

Quick reference for contributors. When you're adding a desktop feature, use this table to decide which epic it belongs to. When in doubt, err on the side of a smaller Epic 5 scope.

---

## Ownership Table

| Concern | Epic 5 (Desktop Shell) | Epic 8 (Sync & API) | Epic 9 (Reliability & Safety) |
|---|:---:|:---:|:---:|
| Tauri desktop app shell | ✓ | | |
| Remote / local server connection flow | ✓ | | |
| Mode indicator UI | ✓ | | |
| Remote-mode no-cache guardrails | ✓ | | |
| Local server launch / stop / monitor (Z131) | ✓ | | |
| Rich tabs and native menu (Z132) | ✓ | | |
| Native notifications (Z13) | ✓ | | |
| Tray / background presence | ✓ | | |
| Global capture | ✓ | | |
| Deep links | ✓ | | |
| Window / workspace restore | ✓ | | |
| File, clipboard, media integrations | ✓ | | |
| Open data folder / one-shot export trigger | ✓ | | |
| Robust HTTP API surface | | ✓ | |
| MCP capability coverage | | ✓ | |
| Real-time update channels | | ✓ | |
| Scope-based data model (Z91) | | ✓ | |
| Cross-device sync design (Z141) | | ✓ | |
| Offline mutation queue with conflict resolution | | ✓ | |
| Backup schedules and integrity checks | | | ✓ |
| Restore / migration tooling | | | ✓ |
| Audit log | | | ✓ |
| Long-term data safety and hardening | | | ✓ |

---

## Rules of Thumb

### Do add to Epic 5 if…
- It improves the desktop shell experience on a single device.
- It operates against a single server (remote or local) with no replication.
- It exposes what's already on the local machine without managing replication.

### Do NOT add to Epic 5 if…
- It requires knowing about more than one Zealot server or device.
- It needs to reconcile conflicting changes from two sources.
- It handles backup scheduling, automated restore, or data integrity guarantees beyond "the local server is running."

---

## Where NOT to Add Caching (Remote Mode)

In fully remote mode the desktop app must behave as a thin client. Do not add:

- Service worker caches for API responses
- IndexedDB stores for item content
- `LazyData<T>` instances that persist to disk
- Search indexes built from server content
- Notification payloads that include item content

All in-memory caches (`LazyData<T>`, component state) are fine — they are cleared when the app closes or the user logs out.

See [desktop-mode-boundaries.md](desktop-mode-boundaries.md) for the full policy.

---

## How to Add a New Desktop Feature

1. **Decide the mode scope.** Does the feature behave differently in remote vs local mode? If yes, gate on `getDesktopMode()` from `packages/engine/src/desktop_mode.ts`.
2. **Check the cache policy.** If the feature loads server data, does it write anything to disk? If yes, is that data item content? If yes, it must be blocked in remote mode (or require explicit user opt-in).
3. **Log safely.** Use `logInfo` / `logError` from `packages/engine/src/log.ts`. Pass only IDs and status codes in the `context` argument — never item titles, content, or attribute values.
4. **Notification payloads.** If the feature triggers a native notification, remote mode payloads must contain only `{ action, item_id }`. Content is fetched after tap.
5. **Update the verification checklist** in [desktop-mode-boundaries.md](desktop-mode-boundaries.md) if your feature introduces a new storage or notification surface.
