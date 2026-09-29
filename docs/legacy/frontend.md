# Frontend (client)

## Stack

- TypeScript
- Vite
- SCSS
- Navigo router
- Luxon for date/time handling
- Custom elements (web components style architecture)

## Entry and App Shell

- Entry file: `client/src/main.ts`
- Root app element: `client/src/app/zealot_app.ts`

Runtime behavior:

1. Check auth status (`/api/account/is_logged_in`).
2. If logged out, render auth modal.
3. If logged in, load user details and render main shell.

## Client Routing

Router setup is in `client/src/features/router/router.ts`.

Main route groups:

- Home and item routes
- Media route
- Planner routes:
  - daily
  - weekly
  - monthly
  - annual
- Types and individual type route
- Analysis route
- Rules route
- Settings screens

## Feature Modules

- `features/item`: item view/editor, relations, attribute editing, comments
- `features/planner`: day/week/month/year planner screens
- `features/types`: item type screens
- `features/media`: media browser screen
- `features/settings`: grouped settings screens
- `features/zealotscript`: editor/parser/serializer command system
- `features/search`: search modal and helpers
- `features/auth`: login/register modal flow

## API Layer

`client/src/api/` modules map directly to backend resources:

- auth
- item
- item_type
- attribute / attribute_kind
- planner
- repeat
- comment
- media
- tracker (exists client-side)

The shared request helper (`client/src/shared/api_helper.ts`) handles:

- credentials-enabled fetch calls
- CSRF warm-up via `/api/health`
- `X-Csrf-Token` header injection for mutating methods
- standard popup error handling

## Build and Serve

- Dev server: `npm run dev` (host enabled in script)
- Production build: `npm run build`
- Docker image serves static output (`/bin/client`) through Nginx

Vite dev proxy forwards `/api` to backend (`http://localhost:8082` by default).
