# Zealot — Product Overview

## What Zealot Is

Zealot is a self-hosted personal wiki and planner. Every piece of information — a note, a task, a goal, a meeting, a habit, a project — is an **item**: a titled document with a freeform body. You give items structure through **types** (user-defined categories like Task, Project, or Meeting) and **attributes** (typed key-value fields such as Status, Priority, or Date). Items link to each other in a hierarchy and through named relationships. The planner is date-driven: any item with a `Date` attribute appears on the daily, weekly, monthly, or annual planner for that date. There is no vendor cloud; your data lives in a SQLite database on your own machine or server.

---

## Core Use Cases

### 1. Planning

The planner shows items by date. Any item — a task, a goal, a daily note — appears on the day its `Date` attribute is set to. There are four planner views: daily, weekly, monthly, and annual.

The planning loop is simple: set a `Date` on any item to put it on the calendar. Move the date to reschedule it. A freeform **Daily Plan** item gives you a scratchpad for the day alongside your scheduled tasks.

```
Daily Planner — 2026-06-05
├── [Task]       Write homepage copy          Status: In Progress
├── [Task]       Review design brief          Status: Not Started
└── [Daily Plan] Plan — 2026-06-05            (freeform scratchpad)
```

> **Screenshot placeholder:** Daily planner view showing a mix of scheduled tasks and a daily plan note.
> _Replace with `docs/screenshots/planner-daily.png` once captured._

---

### 2. Personal Wiki

Any item can be a wiki page. Item bodies use **ZealotScript** — GitHub-flavoured Markdown extended with wiki links, callout blocks, inline math, and more. Type `[[Other Item Title]]` in a body to create a clickable cross-reference to another item. The sidebar navigation tree reflects the parent-child hierarchy of your items.

There is no separate concept of "page" versus "task." A research note and a to-do item are the same kind of thing with different types assigned.

```markdown
## Meeting notes — Design review

Agreed to move the launch date. See [[Website Redesign]] for context.

:::note
Follow up with design team by Friday.
:::
```

> **Screenshot placeholder:** Item editor showing a wiki page body with wiki links and a callout block.
> _Replace with `docs/screenshots/item-wiki-page.png` once captured._

---

### 3. Project Tracking

Projects and tasks are both items. A **Project** item has child **Task** items linked to it via the parent relationship. Each task carries attributes — `Status` (a dropdown: Not Started, In Progress, Complete, Blocked), `Priority`, `Date` — and appears on the planner when scheduled.

The item list view lets you filter by type and sort by attribute, giving you a task list scoped to any project. The analysis dashboard provides 30-day rolling stats: items completed over time, status distribution, priority distribution.

```
Website Redesign  [Project]
├── Write homepage copy   [Task]  Status: In Progress   Date: 2026-06-05
├── Design mockups        [Task]  Status: Not Started
└── Set up staging        [Task]  Status: Not Started
```

> **Screenshot placeholder:** Item list filtered to a project's child tasks, showing Status and Date attribute columns.
> _Replace with `docs/screenshots/project-task-list.png` once captured._

---

### 4. Recurring Work

Two mechanisms handle recurring work.

**Rules engine scheduling:** A rule with a cron trigger runs Lua code on a schedule. Common patterns: create a daily agenda item every morning, generate a weekly review every Sunday, stamp a completion date when a status changes. See the [Rules Engine](./rules-engine.md) for details and a cookbook.

**Planner repeat tracking:** The daily planner has a dedicated section for repeat items — habits and recurring tasks with a per-day status (Complete, Skip, Not Complete). You assign items to the repeat tracker and check them off each day without creating new items.

> **Screenshot placeholder:** Daily planner repeat section showing habit items with per-day status toggles.
> _Replace with `docs/screenshots/planner-repeat.png` once captured._

---

### 5. Knowledge Taxonomy

Types and attributes are entirely user-defined. There are no built-in categories, no mandatory fields, and no fixed hierarchy. You define your own ontology.

Because everything is an item, your taxonomy entries are first-class citizens. A "Project Area" or "Topic" is itself an item with a body, attributes, and links — not just a label or a tag string. You can write notes on it, link to it from other items, and give it its own attributes.

Common patterns people build:
- A flat tag system using the `tag` relationship type
- A topic-based organisation using `topic` links
- A nested hierarchy using parent-child relationships
- Type-scoped views: filter the item list to show only items of type `Meeting` or `Goal`

> **Screenshot placeholder:** Types configuration screen showing a set of user-defined item types.
> _Replace with `docs/screenshots/settings-types.png` once captured._

---

### 6. Automation

The **rules engine** is Zealot's most distinctive feature. Rules are Lua 5.4 scripts that run inside a sandboxed environment. Each rule has a trigger, a script, and an output log.

**Trigger types:**

| Trigger | When it fires |
|---|---|
| Event | When an item is created, updated, or deleted; when a type is assigned; when an attribute is set |
| Schedule | On a cron expression (e.g. `0 8 * * *` for 8 AM daily) |
| Manual | On demand, from the Rules screen |

**Example — stamp completion date when Status is set to "Complete":**

```lua
local item = zealot.event.item
if item.attributes["Status"] == "Complete" then
  zealot.items.set_attribute(item.id, "Completed Date", os.date("%Y-%m-%d"))
end
```

**Example — create a daily agenda item at 8 AM:**

```lua
local today = os.date("%Y-%m-%d")
local id = zealot.items.create({ title = "Daily Plan — " .. today })
zealot.items.set_attribute(id, "Date", today)
zealot.items.assign_type(id, "Daily Plan")
zealot.notify("Created: Daily Plan — " .. today)
```

Full API reference and cookbook: [Rules Engine](./rules-engine.md).

> **Screenshot placeholder:** Rules screen showing a list of active rules and the output log from the last run.
> _Replace with `docs/screenshots/rules-screen.png` once captured._

---

## How Zealot Compares

This section makes no recommendation. The goal is to describe the genuine differences so you can decide.

---

### Notion

Notion is a polished team workspace with a rich block editor, a large template gallery, database views, and Notion AI. It has no setup and works in any browser.

| Choose Zealot if… | Stick with Notion if… |
|---|---|
| You want self-hosted data with no subscription | You share a workspace with a team |
| You want server-side automation via code (Lua) | You want drag-and-drop page building and Notion AI |
| You are comfortable with Git and Docker | You want zero-setup, browser-only access |
| You want an open-source system you can modify or extend | You rely on Notion's template gallery or integrations |

---

### Obsidian

Obsidian stores notes as plain `.md` files in a local vault. It has a large plugin ecosystem, a graph view, and excellent offline support. Your files stay readable in any text editor.

| Choose Zealot if… | Stick with Obsidian if… |
|---|---|
| You want structured attributes and filtering on top of notes | You want plain `.md` files that work in any editor or tool |
| You want a built-in date-driven planner | You depend on specific Obsidian plugins (Dataview, Templater, Excalidraw, etc.) |
| You want built-in automation without adding plugins | You want Obsidian Sync or the published plugin ecosystem |
| You want a single unified model for notes and tasks | You want maximum portability — Obsidian files are just files |

Obsidian's plugin ecosystem is vastly larger than Zealot's. Zealot is a cohesive, opinionated system; Obsidian is a platform you build on top of.

---

### Todoist

Todoist is a focused, reliable task manager with fast capture, natural language scheduling, mobile apps, and team task sharing.

| Choose Zealot if… | Stick with Todoist if… |
|---|---|
| You want tasks to live alongside full wiki pages and notes | You want a polished, low-friction task capture app |
| You want custom attributes beyond labels and priority | You use natural language entry (e.g. "dentist tomorrow at 3pm") |
| You want automation that runs server-side on a schedule | You share tasks with family or a team |
| Notes and tasks should be the same data type | You want a mature, stable mobile app |

---

### Org Mode

Org Mode is a plain-text system built into Emacs. It has an agenda, powerful export formats, decades of stability, and an enormous ecosystem (org-roam, ox-hugo, etc.). For people who live in Emacs, it has no real competition.

| Choose Zealot if… | Stick with Org Mode if… |
|---|---|
| You want a web UI and do not live in Emacs | You are already in Emacs and want maximum text-editing power |
| You want a graphical planner instead of agenda buffers | You want plain text files you can diff, grep, and version-control directly |
| You want automation without writing Emacs Lisp | You want the full Org ecosystem (org-roam, ox-hugo, org-babel, etc.) |

Org Mode is unmatched for people who are already in Emacs. Zealot does not try to compete there.

---

### Plain Wiki (MediaWiki, DokuWiki, etc.)

A traditional wiki is simple, well-understood, and widely deployable. If you just need a shared reference that many people can read, a wiki is hard to beat.

| Choose Zealot if… | Stick with a plain wiki if… |
|---|---|
| You want structured attributes (Status, Date, Priority) on pages | You want zero-friction setup with minimal dependencies |
| You want date-driven planning built into the same tool | Your team is already on a wiki and migration is costly |
| You want automation that can read and write content programmatically | You need a primarily read-only reference that many people can view |

---

## Creative Use Cases

These are concrete ways people use Zealot beyond the generic "notes and tasks" framing. Each builds on the same item + attribute + rules model described above.

---

### Book and reading tracker

Create one item per book and assign it a `Reading` type. A `Status` dropdown attribute (Want to Read → Reading → Finished) tracks progress. Add a `Source` text attribute for the author or URL. Write reading notes directly in the item body using `[[wiki links]]` to connect ideas across books. A simple event rule auto-stamps a `Finished Date` attribute when the status flips to Finished. Filter the item list to `Reading` to see your whole library at a glance.

---

### Weekly review system

A cron rule fires every Sunday evening. It creates a "Week in Review — YYYY-Www" item, queries last week's completed tasks, and pre-populates the body with a summary. Connect Claude via MCP and ask it to write the narrative review from those bullet points — identifying themes, noting what slipped, and suggesting next-week focus areas. The result lives in Zealot as a permanent record.

---

### Goal cascade

An annual `Goal` item holds your high-level intention in its body: the why, the context, the success criteria. Under it, `Milestone` child items mark the major steps. Under each milestone, `Task` child items are the concrete next actions — each with a `Date` and `Status`. The planner surfaces the tasks when scheduled; the wiki hierarchy lets you navigate from a specific task back up to the goal it serves.

---

### Personal research base

One item per paper, article, or source. Each item has a `Source URL` text attribute, a `Status` (Unread → Reading → Processed), and a body with your notes and highlights. Type `[[Concept Name]]` in any body to cross-reference another item — building a graph of connected ideas over time. Full-text search lets you find notes by keyword across your entire library.

---

### Habit dashboard with auto-summaries

Add your recurring habits — exercise, reading, meditation — to the planner's repeat tracker. Each day you mark them Complete, Skip, or Alternate. A cron rule runs every Sunday and uses the Lua API to read that week's repeat entries, tally the completion rate for each habit, and write a comment on the habit item with the weekly score. Over time, each habit item accumulates a longitudinal log you can scroll through.

---

### AI-assisted journaling

Use `add_comment` (via the MCP or API) to log quick notes throughout the day against a "Journal" item or against the items you're actively working on. At the end of the week, ask Claude via MCP to read your comments from the past seven days and summarise recurring themes, mood patterns, or things that went well and poorly. The raw log stays in Zealot; the synthesis lives in a new "Weekly Reflection" item created by the agent.

---

## Limitations

**Maturity.** Zealot is under active development. The backend API is stable; the web UI is being incrementally rewritten. Some screens described in this documentation may look different or may not yet be fully implemented. Specific features including the planner repeat tracking UI and the analysis dashboard are works-in-progress. The tool is viable for daily use, but expect to encounter rough edges.

**No pre-built distribution.** There is no hosted version, no package manager install, and no pre-built Docker image published to a registry. Running Zealot today requires cloning the repository and building from source via `npm run docker`. The first build takes a few minutes; subsequent starts are fast. It is not technically difficult, but it is more steps than installing a conventional app.

**Schema design is your job.** Zealot ships with no built-in item types or attributes. Before the planner and type filters are useful, you need to define your own types (Task, Project, Meeting, etc.) and attribute kinds (Status, Date, Priority, etc.). The [Quickstart](./quickstart.md) walks through this in about ten minutes. Users who dislike upfront configuration will find this friction.

**Automation requires Lua.** The rules engine is powerful, but using it requires comfort with Lua 5.4. Typical automation — stamping dates, generating agenda items, sending digests — is 5–20 lines of straightforward Lua. The [Rules Engine](./rules-engine.md) has a cookbook with ready-to-paste examples. But if you want automation and have no programming background, the rules engine will be an obstacle.

**Single-user first.** Zealot is designed for personal use. There is no built-in multi-user collaboration, shared editing, or team workspace model. Multiple accounts can exist on one instance and the API is accessible externally, but there is no sharing or co-editing model today.

---

## What to Read Next

- [Quickstart](./quickstart.md) — install Zealot and build your first system step by step
- [ZealotScript](./zealotscript/README.md) — full markup syntax reference
- [Rules Engine](./rules-engine.md) — automation with Lua: triggers, API reference, cookbook
- [MCP integration](./mcp-system-prompt.md) — use Zealot with LLM agents
