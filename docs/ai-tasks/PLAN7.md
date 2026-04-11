# Plan: Fix 26 Failing Playwright Tests

## Context

The Playwright test suite has 26 failing tests across auth, items, comments, types, search, and navigation specs. The failures fall into distinct root-cause buckets:

- **2 auth tests** fail because error messages never reach the login/register views
- **1 items test** fails because no "Home" item exists in the test database
- **1 items test** fails because HTML5 form validation blocks custom blank-title handling
- **~22 remaining tests** fail because `POST /api/item/` and `POST /api/item_type/` return 404

---

## Root Causes & Fixes

### Fix 1 — Auth error propagation (`main_screen.ts`)

**Tests fixed:** "login with wrong password shows error", "register with duplicate username shows error"

**Problem:** `on_login_attempt` and `on_register_attempt` in `main_screen.ts` wrap async calls in try-catch, call `Popups.add_error()`, and swallow the error. `login_view.ts` and `register_view.ts` catch the thrown error and set `#error_message.innerText`, but they never receive the error because `main_screen.ts` already consumed it.

**File:** `packages/ui/src/screens/main_screen.ts`

Remove the try-catch wrappers in both callbacks and let errors propagate:
```typescript
// BEFORE
on_login_attempt: async (dto) => {
    try {
        const account = await this.data!.AuthAPI.Basic.login(dto);
        if (account) { Events.emit('to_app'); }
    } catch (e) {
        Popups.add_error((e as Error).message ?? 'Login failed.');
    }
},

// AFTER
on_login_attempt: async (dto) => {
    const account = await this.data!.AuthAPI.Basic.login(dto);
    if (account) { Events.emit('to_app'); }
},
```
Same change for `on_register_attempt`.

---

### Fix 2 — Create "Home" item in test setup (`auth.setup.ts`)

**Tests fixed:** "home page loads the Home item"

**Problem:** `auth.setup.ts` registers and logs in a user but never creates a "Home" item. The navigator immediately calls `GET /api/item/title/Home`, which returns 404, and the app shows "That item doesn't exist." instead of the Home item screen.

**File:** `test/playwright/tests/auth.setup.ts`

After the login succeeds and the `web-client` is visible, add:
```typescript
// Create the Home item required by the home-page tests
const homeRes = await request.post('/api/item/', {
    data: { title: 'Home', content: '' },
});
expect(homeRes.ok()).toBeTruthy();
```

---

### Fix 3 — Remove `required` from title input (`add_item_modal.ts`)

**Tests fixed:** "create item with blank title shows error"

**Problem:** `<input name="title" type="text" required>` triggers the browser's built-in constraint validation. When the title is empty and the form is submitted, the browser intercepts the submit event and displays its own tooltip — the `submit` event is never fired, so the `submit()` method never runs, and `setError('Title is required.')` is never called. The error `div[data-role="error"]` stays `hidden`.

**File:** `packages/ui/src/common/add_item_modal.ts` — line 50

```html
<!-- BEFORE -->
<input name="title" type="text" required>

<!-- AFTER -->
<input name="title" type="text">
```

The modal already performs its own blank-title check in `submit()` (lines 134–139), so the `required` attribute is redundant.

---

### Fix 4 — Investigate `POST /api/item/` and `POST /api/item_type/` returning 404

**Tests affected:** ~22 tests — all `createItem`-dependent tests (comments, navigation, search, items create/edit/delete), plus types tests that call `POST /api/item_type/`.

**Observed symptoms:**
- `createItem failed: 404` in the Playwright fixture
- The navigation pane shows `Failed to get /api/item/ (status 404)` on every page
- Both GET and POST to `/api/item/` return 404
- Auth endpoints (`/api/auth/register`, `/api/auth/login`) work fine

**Backend routing (should be correct):**
- `mod.rs`: `.nest("/item", item::routes(state.clone()))`
- `item.rs`: `.route("/", get(get_root_items).post(add_item))`
- nginx: `location /api/ { proxy_pass http://server:8456/; }` strips `/api` prefix correctly

**Likely causes (in order of probability):**
1. **Stale Docker image** — a previous image named `zealot-server:playwright` is cached and doesn't include the item routes. Despite `compose up --build`, Docker layer caching may have skipped re-compilation. Fix: `docker rmi zealot-server:playwright zealot-web:playwright` then re-run.
2. **Backend not running during local dev** — if someone ran `cd test/playwright && npx playwright test` without `run.sh`, Vite started but Rust backend didn't. Vite proxy behavior on connection-refused may produce 404. Fix: always use `npm run test:playwright` from the repo root.
3. **Axum 0.8 trailing-slash routing edge case** — `nest("/item", router_with_route("/", handler))` might register the route at `/item` not `/item/`. Investigate by adding a `.route("/item/", ...)` fallback or using `.route("/item", ...)` without trailing slash in the API client.

**Investigation steps during implementation:**
```sh
# 1. After `run.sh` starts the stack, test the API directly:
curl -v -X POST http://127.0.0.1:18081/api/item/ \
  -H "Content-Type: application/json" \
  -d '{"title":"test","content":""}' \
  -b session_id=<token>

# 2. Check backend logs for routing info:
docker logs zealot-playwright_server_1 2>&1 | head -50

# 3. Force a clean image rebuild:
docker rmi zealot-server:playwright zealot-web:playwright 2>/dev/null || true
npm run test:playwright
```

---

### Fix 5 — Types test strict-mode violation (`.tool-error` ambiguity)

**Tests fixed:** "create type with blank name shows error" (types:46)

**Problem:** This test uses `locator('.tool-error')` which matches 2 elements — the navigation pane's API error `<p class="tool-error">Failed to get /api/item/ (status 404)</p>` AND the type form's validation error. The locator is in strict mode.

**Fix A (preferred):** Fixing the `/api/item/` 404 (Fix 4) removes the nav error, leaving only 1 `.tool-error` element — test passes without a code change.

**Fix B (test-side fallback, only if Fix 4 is not yet resolved):** Scope the selector to the settings screen:
```typescript
// In types.spec.ts line 54:
await expect(page.locator('settings-screen .tool-error')).toContainText('Type name is required.');
```

---

## Execution Order

1. Apply Fixes 1, 2, 3 (code changes — safe to do first)
2. Run `npm run test:playwright` to get fresh failure output
3. Examine which tests still fail and use investigation steps from Fix 4
4. Apply Fix 4 based on findings
5. Apply Fix 5B only if Fix 4 can't be resolved immediately

---

## Critical Files

| File | Change |
|---|---|
| `packages/ui/src/screens/main_screen.ts` | Remove try-catch in auth callbacks |
| `test/playwright/tests/auth.setup.ts` | Add POST /api/item/ to create Home item |
| `packages/ui/src/common/add_item_modal.ts` | Remove `required` attribute from title input |
| `test/playwright/tests/types.spec.ts` | Fallback scope fix if needed |

## Verification

After all fixes, run:
```sh
npm run test:playwright
```
Expected outcome: ≥ 43 → 69 passing. Any remaining failures should be isolated to environment/Docker issues, not code bugs.
