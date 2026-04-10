# TASK-025 Comments View

## Summary
- Build a single reusable `CommentsView` custom element for the two supported scopes only: planner day and item detail.
- Keep the existing backend contract as-is: use the real `/api/comment/...` routes, not the stale pluralized rewrite docs, and do not change Rust/backend code.
- Use local component state for load/create/edit/delete so [daily_planner_screen.ts](/home/alexander/Projects/zealot/packages/ui/src/screens/daily_planner_screen.ts) and [item_screen.ts](/home/alexander/Projects/zealot/packages/ui/src/screens/item_screen.ts) only mount the view and pass scope.

## Key Changes
- Add [comments_view.ts](/home/alexander/Projects/zealot/packages/ui/src/views/comments_view.ts) with `init(config)` and a scope union: day scope gets a `DateTime`, item scope gets an `itemId`.
- The component owns loading, empty state, error state, draft state, inline edit state, and in-memory comment list updates. Initial load reads comments once; create pushes to the end of the list; edit replaces content in place; delete removes the row locally.
- Render each row with displayed author, formatted timestamp, content, and row actions. Inline edit swaps content text for a textarea with `Save` and `Cancel`. Delete uses the existing `ConfirmDialog`; request failures use `Popups.add_error`.
- Add a bottom composer shared across both scopes. Item scope posts with the current local datetime automatically. Day scope includes an explicit `time` input, defaulted to the current local time, and combines that time with the scoped planner date when posting.
- Reuse shared helpers inside the comments module for timestamp formatting/parsing, message extraction, current-account lookup, and scope-to-request mapping so day/item screens do not duplicate comment logic.
- Integrate the view into the daily planner as a new `Comments` planner section and into the item screen after the related/children collections.
- Export/register the new element from the UI package and add a dedicated shared stylesheet for comment UI in the content package so styling is not split across planner/item screen styles.

## Public Interfaces
- `CommentsViewConfig`:
  - `{ scope: { kind: 'day'; date: DateTime } }`
  - `{ scope: { kind: 'item'; itemId: number } }`
- `CommentAPI` method names stay the same, but its internal serialization/parsing is normalized to the real backend wire format:
  - parse returned timestamps from SQL-style `"yyyy-MM-dd HH:mm:ss"` strings into `Comment.Timestamp`
  - serialize create/update timestamps back to that same string format before calling `/comment/...`
- `CommentsView` is the only public UI entry point for comment CRUD; screens should not call `CommentAPI` directly for comment rendering anymore.

## Test Plan
- Run `npm run typecheck -w @zealot/api`.
- Run `npm run typecheck -w @zealot/ui`.
- Run `npm run build:web`.
- Manual item-scope smoke test: load existing comments, create a comment, inline-edit it, delete it, verify local UI updates without a full screen reload.
- Manual day-scope smoke test: load comments for a planner date, create a comment with a chosen time, verify timestamp display/order, edit content, delete with confirmation, and verify the empty state when the last comment is removed.
- Manual regression check: item screen and daily planner still render when comment loading fails, with the error isolated to the comments section.

## Assumptions And Defaults
- The rewrite docs are stale on comment routes; implementation targets the existing singular `/api/comment` backend.
- The backend exposes no per-comment author or ownership metadata. The UI will display the current authenticated account username as the author label for all loaded comments.
- Because comments are account-scoped in the current backend, edit/delete actions are shown for all loaded comments.
- The legacy unscoped item-picker mode from the old comments component is out of scope; TASK-025 only supports the two explicit usage sites.
