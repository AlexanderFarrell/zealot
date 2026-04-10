# TASK-026: Tracker API & Tracker Entry UI

## Context
The original client had a tracker feature for logging a numeric level + comment per item per day (e.g. mood, energy, pain level). The `TrackerAPI` was present in the original but was never integrated into `ZealotAPI`.

Run `git diff master -- client/src/api/tracker.ts` to see the original API client. Also search the original screens that used tracker data.

## Goal
Implement the Tracker API client and wire tracker entry UI into relevant screens.

## API Endpoints
- `GET /api/tracker/day/:date` — get all tracker entries for a day
- `POST /api/tracker` — add a tracker entry `{ item_id, timestamp, level, comment }`
- `DELETE /api/tracker/:tracker_id` — delete a tracker entry

## Requirements

### API Client
- Create or update `packages/api/src/tracker.ts` with the three endpoints
- Add `TrackerAPI` to `ZealotAPI` in `packages/api/src/index.ts`

### UI: Tracker Entry Widget
- Small widget showing a numeric level input (1–10 or similar range) and optional comment text
- Used on the daily planner screen (TASK-021) alongside items that have tracker-type attributes
- Add/delete tracker entries inline

## Files Likely Involved
- `packages/api/src/tracker.ts`
- `packages/api/src/index.ts`
- `packages/ui/src/` (new tracker entry widget)
- `packages/domain/src/` (add `TrackerEntry` domain type if not present)
