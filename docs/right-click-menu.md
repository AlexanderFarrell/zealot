# Right-Click Context Menu — Planning Doc

## Goal

Add a global context menu system that surfaces relevant actions based on what element the user right-clicks. The menu should be a single global overlay (one `<context-menu>` custom element mounted at the root), populated dynamically by whichever component registered actions for the clicked target.

---

## Architecture Sketch

- **`packages/engine/src/ui/context_menu.ts`** — the global overlay element and registration API.
- Components call `registerContextTarget(element, () => ContextAction[])` to opt a specific element into the menu.
- On any `contextmenu` event, the engine walks up from `event.target` collecting actions from the nearest registered ancestor, then renders the menu at the cursor position.
- The menu auto-closes on `click`, `Escape`, or scroll outside it.
- `ContextAction` type: `{ label: string; icon?: string; onClick: () => void; danger?: boolean }`.

---

## Surfaces & Proposed Actions

Below is a list of each clickable surface in the app, and proposed right-click actions for each. **Please review, add, remove, or adjust before we start implementing.**

---

### 1. Item Card (`item-card` in `item_card_list.ts`)

Item cards appear in: the Nav Tree children list, planner card lists, the Item Screen's Children/Related sidebar sections.

| Action | Notes |
|---|---|
| Open | Navigate to the item (same as left-click) |
| Open in new tab | `window.open(...)` |
| Open in new window |  |
| Copy link | Copy `/item/<title>` URL to clipboard |
| Set parent → (current item) | Only in child-card context; reassigns parent |
| Unlink from parent | Removes the parent relationship |
| Delete | Confirm dialog, then delete |

---

### 2. Nav Tree Node (`nav-tree-node-view` title button)

| Action | Notes |
|---|---|
| Open | Navigate to item |
| Open in new tab | |
| Open in new window |  |
| Copy link | |
| New child item | Opens the add-item flow with this item as parent |
| Expand all children | Eagerly loads and expands the subtree |
| Delete | Confirm + delete |

---

### 3. Item Screen — the current item itself (`item-screen`)

Right-click anywhere on the item page outside of editable fields.

| Action | Notes |
|---|---|
| Copy link | Copy current URL |
| Open in new tab | |
| Open in new window |  |
| Copy as Markdown | (Already available via the download button) |
| Download as PDF | |
| Download as DOCX | |
| Manage types | Opens `AssignTypeModal` |
| Paste template | Opens `PasteTemplateModal` |
| Delete item | Confirm dialog |

---

### 4. Item Screen — Type badges (`tool-badge` in `renderTypes`)

| Action | Notes |
|---|---|
| Open type screen | Navigate to type |
| Remove this type from item | Calls unassign |

---

### 5. Item Table Row (`item-table-view`)

Rows in the type screen's items table.

| Action | Notes |
|---|---|
| Open item | Navigate |
| Open in new tab | |
| Open in new window |  |
| Copy link | |
| Delete item | Confirm + delete |

---

### 6. Planner Card List (`planner_shared.ts` → `mountPlannerCardList`)

Items listed in daily/weekly/monthly/annual planners.

| Action | Notes |
|---|---|
| Open item | Navigate |
| Open in new tab | |
| Copy link | |
| Set status → … | Submenu or quick-pick: To Do / Working / Complete / Hold / etc. |
| Reschedule → today | Sets Date attribute to today |
| Reschedule → tomorrow | |
| Remove from planner | Clears the planner attribute (Date / Week / Month / Year) |
| Delete item | Confirm + delete |

---

### 7. Planner Header Nav Buttons (daily/weekly/monthly/annual)

These already have `onDrop` handlers. A right-click menu is probably not useful here. **Skip unless you want to add something.**

---

### 8. Search Results (`search_tool_view.ts`)

| Action | Notes |
|---|---|
| Open | Navigate |
| Open in new tab | |
| Open in new window |  |
| Copy link | |

---

### 9. Calendar Tool (`calendar_tool_view.ts`)

A calendar day cell, if items are shown there.

| Action | Notes |
|---|---|
| Go to day in planner | Navigate to that day's planner view |
| New item for this day | Create item with `Date` set |

---

### 10. Types Screen (`types_screen.ts`) — type row

| Action | Notes |
|---|---|
| Open type | Navigate to type screen |
| Delete type | Confirm + delete (non-system types only) |

---

## Open Questions

1. **Submenus** — "Set status →" and similar hierarchical actions would need a submenu layer. Worth implementing now, or defer to a flat list with explicit labels ("Set status: Complete")?
2. **Keyboard accessibility** — Menu should be navigable with arrow keys and Enter. Include in initial implementation?
3. **Icon column** — Do we want icons next to each menu item, or text-only?
4. **Danger styling** — Delete actions should be visually distinguished (red text). Keep a `danger` flag on `ContextAction`?
5. **Registration approach** — A `registerContextTarget(el, () => actions[])` callback means actions are computed lazily at open time, which lets them reflect current state. Is this the right model, or do you prefer event-based (fire a `contextmenu` custom event and let parents respond)?
