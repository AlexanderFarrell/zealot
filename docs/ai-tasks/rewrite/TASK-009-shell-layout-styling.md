# TASK-009: Complete Shell Layout & Styling

## Context
`apps/web/src/web_client.ts` defines the top-level shell but `header_bar.ts`, `center_content.ts`, and `side_bar.ts` in `packages/ui/src/` are stubs. The original client had a full responsive layout with SCSS.

Run `git diff master -- client/src/app/shell/` and `git diff master -- client/src/assets/styles/` to see the original layout and styles.

## Goal
Flesh out the shell into a complete, styled layout.

## Requirements

### Layout
- Top header bar: app title/logo, navigation actions, user menu
- Left sidebar: tab strip (`side_buttons.ts`) controlling which sidebar view is shown (nav tree, search, calendar, attributes)
- Main content area: fills remaining space, renders the current screen
- Mobile layout: collapsible sidebar, hamburger menu, mobile title bar

### Styling
Port the SCSS from the original client's `assets/styles/`:
- `constants.scss` — design tokens (colors, spacing, typography)
- `common.scss` — base element resets and shared utilities
- `layout.scss` — shell grid/flex layout
- `main.scss` — root app styles
- `screens.scss` — per-screen styles
- `tags.scss` — custom element tag styles

### Responsive
- Sidebar collapses on narrow viewports
- Content area scrolls independently of the sidebar
- Touch-friendly tap targets on mobile

## Files Likely Involved
- `packages/ui/src/header_bar.ts`
- `packages/ui/src/center_content.ts`
- `packages/ui/src/side_bar.ts`
- `packages/ui/src/side_buttons.ts`
- `packages/content/src/` (CSS assets)
- `apps/web/src/web_client.ts`
