# Zealot Desktop Mode Security and Offline Boundaries

This document defines the security and data-storage boundaries for Zealot Desktop's two operating modes. It is the authoritative reference for Epic 5 desktop implementation decisions. See also [desktop-epics-boundary.md](desktop-epics-boundary.md) for the Epic 5 / 8 / 9 scope split.

---

## Two Modes

| | Fully Remote Mode | Local Server Mode |
|---|---|---|
| **Server location** | Remote Zealot server (any URL) | Local Zealot process on the same machine |
| **Primary trust concern** | Don't leave sensitive data on the device | The local server/DB is authoritative on this device |
| **Offline use** | No — requires network connectivity to the remote server | Yes — the local server works without internet |
| **Sync** | N/A | Future: Epic 8 scope-based sync with a remote server |

---

## Fully Remote Mode

### What may be stored locally

| Key | Store | Justification |
|---|---|---|
| `zealot_settings` | localStorage | Theme and UI preferences only — no item content |
| `zealot_serverUrl` | localStorage | Needed across app restarts |
| `zealot_apiKey` | Tauri secure keychain (preferred) / localStorage (interim) | Auth credential — no item content |
| `zealot_apiKeyId` | Tauri secure keychain (preferred) / localStorage (interim) | Needed to revoke the key on logout |

### What must NOT be stored locally

- Item titles, content, attributes, or comment bodies
- Search indexes or query result caches
- Notification payloads that include item content (titles, bodies, attribute values)
- `LazyData<T>` instances backed by disk — all in-memory caches are acceptable but must be cleared on app close / logout
- Response body caches (no service worker cache, no IndexedDB response store)

### Credentials and session protection

- API key is the auth mechanism for non-browser clients (see `packages/engine/src/api/api_helper.ts` — `SetApiKey` / `X-Api-Key` header).
- On logout, call `logout()` (defined in `apps/mobile/src/mobile_core.ts` and its desktop equivalent) which: revokes the key server-side, clears `SetApiKey(null)`, and removes all `localStorage` entries.
- When Tauri's secure keychain plugin is available, migrate `zealot_apiKey` / `zealot_apiKeyId` out of localStorage into the keychain. Until then, localStorage is acceptable as an interim.
- CSRF tokens are session-scoped cookies and are cleared automatically by the browser/webview on close; they are not a local-storage concern.

### Crash recovery and unsaved drafts

- **Default: off.** No draft content is written to disk in remote mode.
- A user may opt in to transient crash recovery via Settings (`draft_recovery_remote: boolean`, stored in `zealot_settings` — the setting itself contains no item content).
- If opt-in is active, drafts must be stored in `sessionStorage` only (cleared on webview exit), never in `localStorage` or a file, and must be purged on successful save or discard.

### Notification payload safety

- When native notifications are implemented (Epic 5 — Z13), notification payloads in remote mode must contain only the item ID and a generic action label.
- Example safe payload: `{ "action": "item_updated", "item_id": 42 }`.
- Example unsafe payload (must NOT be sent): `{ "title": "Buy groceries", "body": "milk, eggs, bread" }`.
- The desktop app fetches the item content from the server after the user taps the notification.

---

## Local Server Mode

### Data directory

| Platform | Default path |
|---|---|
| Linux | `~/.local/share/zealot/` |
| macOS | `~/Library/Application Support/zealot/` |

The desktop app must display this path to the user and provide an "Open data folder" action. It must not hard-code any other location.

### Ownership

- The SQLite database (`zealot.db`) and media directory are owned and managed exclusively by the local Zealot server process.
- The desktop app must not read or write these files directly. All access goes through the HTTP API (same as remote mode).
- The desktop app may launch, stop, and monitor the local server process (see Z131), but treats the server as the data authority.

### Backup and export

- Epic 5 scope: expose "Open data folder" and trigger a server-initiated export/backup action via the API.
- Epic 9 scope: automated backup schedules, backup integrity checks, migration tooling, and restore flows. Do not absorb these into Epic 5.

### Unsaved drafts

- Transient crash recovery is allowed in local mode.
- Drafts may be stored in `sessionStorage` or a temp file managed by the local server. They must be purged after a successful save.
- There is no security concern about draft content living on the local machine (the full database is already local).

### Future sync

- The local server is the authoritative store on this device until Epic 8 scope-based sync is implemented.
- Do not implement any mutation queue, conflict resolution, or replication logic in Epic 5. If you are tempted to, it belongs in Epic 8.

---

## Log Safety (Both Modes)

Zealot logs must never contain sensitive item content. This applies to both the TypeScript frontend and the Rust backend.

### Frontend (`packages/engine/src/log.ts`)

Use the `logInfo` / `logError` wrappers defined in `packages/engine/src/log.ts`. The `context` parameter accepts only IDs and status codes:

```ts
// Correct
logError('Failed to fetch item', { item_id: 42, status: 404 });

// WRONG — never do this
logError('Failed to fetch item', { title: item.title, content: item.content });
```

### Rust backend

Use the `tracing` crate. Field values must be IDs, counts, or status codes:

```rust
// Correct
tracing::warn!(item_id = id, status = 404, "item not found");

// WRONG
tracing::warn!(title = %item.title, "item not found");
```

---

## Mode Indicator

A visible `<zealot-mode-indicator>` component must be present in the desktop shell header when running in either mode. It is hidden when running as a plain web app (mode is `null`).

- Remote mode: shows "Remote — {hostname}" pill
- Local mode: shows "Local" pill

Implementation: `packages/ui/src/common/desktop_mode_indicator.ts`
Mode state: `packages/engine/src/desktop_mode.ts` — `setDesktopMode()` called once at desktop app init.

---

## Mode Switching

If the user switches from local to remote (or vice versa), the app must:

1. Call `logout()` to clear all credentials and API state.
2. Clear all in-memory `LazyData` caches.
3. Call `setDesktopMode(newMode)` with the new mode.
4. Navigate to the connection setup screen.

There is no automatic migration of data between modes.

---

## Verification Checklist

After implementing any Epic 5 desktop feature, verify:

- [ ] Mode indicator is visible and shows the correct mode label.
- [ ] After a remote-mode session, inspect `localStorage`: only `zealot_settings`, `zealot_serverUrl`, `zealot_apiKey`, `zealot_apiKeyId` are present. No item titles, content, or search results.
- [ ] After `logout()`, all four keys above are absent from `localStorage`.
- [ ] Notification payload (when Z13 is implemented): does not include item titles or content in remote mode.
- [ ] Local-mode data directory path matches the platform default above.
- [ ] Log output: `grep` for known item title strings — expect zero matches.
- [ ] Draft recovery is off by default in remote mode; check `zealot_settings.draft_recovery_remote` is absent or `false`.
