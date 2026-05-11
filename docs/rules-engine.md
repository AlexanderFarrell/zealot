# Zealot Rules Engine

The rules engine lets you automate Zealot using **Lua scripts**. Scripts can respond to events (an item is created, a type is assigned), run on a schedule (every morning at 8 AM), or run on demand. Through the `zealot` API, scripts can read and write items, set attributes, add comments, and generate content.

---

## Table of contents

1. [Quick start](#quick-start)
2. [How rules work](#how-rules-work)
3. [Trigger types](#trigger-types)
4. [Cron schedule syntax](#cron-schedule-syntax)
5. [The `zealot` API](#the-zealot-api)
   - [zealot.items](#zealotitems)
   - [zealot.comments](#zealotcomments)
   - [Utilities](#utilities)
   - [zealot.event](#zealothevent)
6. [Item table structure](#item-table-structure)
7. [Writing safe rules](#writing-safe-rules)
8. [Building content with ZealotScript](#building-content-with-zealotscript)
9. [Cookbook](#cookbook)

---

## Quick start

1. Open the **Rules** screen from the sidebar.
2. Click **New Rule**.
3. Give it a name: `Hello World`.
4. Set the trigger to **Manual**.
5. Paste this script:

```lua
zealot.notify("Rules are working!")
```

6. Click **Save**, then **Run Now**.

The output panel shows `Rules are working!`. That's it — you've written your first rule.

---

## How rules work

A rule has three parts:

- **Trigger** — when the script runs (event, schedule, or manual)
- **Script** — a Lua 5.4 program that runs inside a safe sandbox
- **Output** — anything passed to `zealot.notify()` is captured and displayed in the Rules screen after the run

Each time a rule fires, it gets a **fresh Lua environment**. There is no persistent state between runs — use item attributes or content to store state that needs to survive across executions.

Rules run asynchronously in the background. They do not block the user interface.

---

## Trigger types

### Event triggers

Fire immediately when something happens in Zealot.

| Trigger | Fires when… |
|---|---|
| `on_item_create` | A new item is created |
| `on_item_update` | An item's title or content is changed |
| `on_item_delete` | An item is deleted |
| `on_comment_add` | A comment is added to any item |
| `on_type_assign` | A type is assigned to an item |
| `on_type_unassign` | A type is removed from an item |
| `on_attribute_set` | An attribute value is set on any item |

For `on_type_assign`, `on_type_unassign`, and `on_attribute_set` you can optionally filter by a specific type name or attribute key. Leave the field blank to match any.

When an event trigger fires, `zealot.event` is populated with details about what happened. See [zealot.event](#zealothevent) below.

### Scheduled triggers

Run on a time-based schedule.

| Trigger | Configuration |
|---|---|
| **Cron** | A 5-field cron expression (see [cron syntax](#cron-schedule-syntax)) |
| **Interval** | A number of seconds between runs |

The scheduler checks every 60 seconds. A cron rule fires once per matching minute. An interval rule fires when `now − last_run ≥ interval`.

### Manual trigger

Runs only when you click **Run Now** in the Rules screen. Use this for:

- One-off bulk operations (migrate attribute keys, backfill data)
- Reports you want to generate on demand
- Testing a script before scheduling it

---

## Cron schedule syntax

Cron expressions have five space-separated fields:

```
┌─────── minute       (0–59)
│ ┌───── hour         (0–23)
│ │ ┌─── day of month (1–31)
│ │ │ ┌─ month        (1–12)
│ │ │ │ ┌ day of week (0–7, 0 and 7 = Sunday)
│ │ │ │ │
* * * * *
```

Use `*` to mean "every". Use `/n` to mean "every n-th". Use `,` for lists.

### Common patterns

| Expression | Meaning |
|---|---|
| `0 8 * * *` | Every day at 8:00 AM |
| `0 9 * * 1` | Every Monday at 9:00 AM |
| `0 20 * * 0` | Every Sunday at 8:00 PM |
| `0 0 * * *` | Every day at midnight |
| `0 23 * * *` | Every day at 11:00 PM |
| `0 6 * * 1-5` | Weekdays at 6:00 AM |
| `30 7 * * 1,3,5` | Mon, Wed, Fri at 7:30 AM |
| `0 0 1 * *` | First day of every month |
| `*/15 * * * *` | Every 15 minutes |

---

## The `zealot` API

All Zealot functionality is available through the `zealot` global table. Every function that reads or writes data is safe — it runs as the rule's owner and can only access that account's data.

### `zealot.items`

#### Read operations

**`zealot.items.get(id)`**  
Returns the item with the given ID, or `nil` if not found.
```lua
local item = zealot.items.get("abc123")
if item then
    zealot.notify(item.title)
end
```

**`zealot.items.get_by_title(title)`**  
Returns the item with exactly that title, or `nil`.
```lua
local dashboard = zealot.items.get_by_title("Project Dashboard")
```

**`zealot.items.find_by_type(type_name)`**  
Returns an array of all items that have the given type assigned.
```lua
local tasks = zealot.items.find_by_type("Task")
for _, task in ipairs(tasks) do
    zealot.notify(task.title)
end
```

**`zealot.items.search(term)`**  
Returns items whose titles match the search term (same as the search sidebar).
```lua
local results = zealot.items.search("meeting")
```

**`zealot.items.filter(filters)`**  
Returns items matching a set of attribute filters. Each filter is a table with `key`, `op`, and `value`.

```lua
local items = zealot.items.filter({
    { key = "Status",   op = "eq",  value = "In Progress" },
    { key = "Priority", op = "eq",  value = "High" },
})
```

Supported `op` values:

| Op | Meaning |
|---|---|
| `eq` | Equal |
| `ne` | Not equal |
| `gt` | Greater than |
| `lt` | Less than |
| `gte` | Greater than or equal |
| `lte` | Less than or equal |
| `ilike` | Case-insensitive substring match |

#### Write operations

**`zealot.items.create(title, content?, opts?)`**  
Creates a new item. Returns the created item table.

```lua
local item = zealot.items.create("My New Item", "# Content\n\nHello.")
```

With options:
```lua
local item = zealot.items.create("Meeting Notes", "", {
    types = { "Meeting" },
})
```

**`zealot.items.update(id, opts)`**  
Updates an item. Returns the updated item table. Provide only the fields you want to change.

```lua
zealot.items.update(item.id, {
    title   = "New Title",
    content = "# Updated\n\nNew content.",
})
```

**`zealot.items.set_attribute(id, key, value)`**  
Sets a single attribute on an item. Returns `true` on success.

```lua
zealot.items.set_attribute(item.id, "Status", "Complete")
zealot.items.set_attribute(item.id, "Priority", "High")
zealot.items.set_attribute(item.id, "Date", "2026-05-15")
```

**`zealot.items.assign_type(id, type_name)`**  
Assigns a type to an item. Returns `true` on success.

```lua
zealot.items.assign_type(item.id, "Meeting")
```

**`zealot.items.delete(id)`**  
Deletes an item permanently. Returns `true` on success. Use with care.

```lua
zealot.items.delete(item.id)
```

---

### `zealot.comments`

**`zealot.comments.add(item_id, content)`**  
Adds a comment to an item. Returns the created comment table.

```lua
zealot.comments.add(item.id, "Reminder: due tomorrow.")
```

---

### Utilities

**`zealot.notify(message)`**  
Appends a string to the rule's output log. Visible in the Rules screen after the run. Use this to report what the rule did.

```lua
zealot.notify("Processed 5 items.")
zealot.notify("Skipped: " .. item.title)
```

**`zealot.log(message)`**  
Alias for `zealot.notify`.

**`zealot.now`**  
The ISO-8601 datetime string of when the rule started executing.
```lua
-- e.g. "2026-05-11T09:00:00+00:00"
zealot.notify("Running at: " .. zealot.now)
```

**`zealot.date`**  
Today's date as a `YYYY-MM-DD` string.
```lua
-- e.g. "2026-05-11"
zealot.items.set_attribute(item.id, "Date", zealot.date)
```

---

### `zealot.event`

Available only in event-triggered rules. Contains details about what triggered the rule. Is `nil` when running in scheduled or manual context.

Always check `zealot.event` before using it in a shared script:
```lua
if zealot.event then
    zealot.notify("Triggered by: " .. zealot.event.kind)
end
```

#### Fields by event kind

**`on_item_create` and `on_item_update`**
```
zealot.event.kind    -- "on_item_create" or "on_item_update"
zealot.event.item    -- the full item table (see Item table structure below)
```

**`on_item_delete`**
```
zealot.event.kind      -- "on_item_delete"
zealot.event.item_id   -- string ID of the deleted item
```

**`on_comment_add`**
```
zealot.event.kind        -- "on_comment_add"
zealot.event.item_id     -- string ID of the item the comment was added to
zealot.event.content     -- the comment text
zealot.event.timestamp   -- ISO datetime string
```

**`on_type_assign` and `on_type_unassign`**
```
zealot.event.kind        -- "on_type_assign" or "on_type_unassign"
zealot.event.item        -- the full item table
zealot.event.type_name   -- the type that was assigned or removed
```

**`on_attribute_set`**
```
zealot.event.kind           -- "on_attribute_set"
zealot.event.item           -- the full item table (with the new attribute value already set)
zealot.event.attribute_key  -- the key of the attribute that was changed
```

---

## Item table structure

Every item returned by the `zealot.items` API has these fields:

```
item.id           -- string, the item's unique ID
item.title        -- string
item.content      -- string, the item's ZealotScript (markdown) content
item.attributes   -- table, keys are attribute names, values are the attribute values
item.types        -- array of strings (type names assigned to this item)
item.links        -- array of { id: string, relationship: string } tables
```

### Reading attributes

```lua
local status   = item.attributes["Status"]    -- nil if not set
local due_date = item.attributes["Date"]
local priority = item.attributes["Priority"]

if status == "Complete" then
    zealot.notify(item.title .. " is done.")
end
```

### Reading types

```lua
for _, type_name in ipairs(item.types) do
    zealot.notify(item.title .. " has type: " .. type_name)
end

-- Check if an item has a specific type
local has_meeting = false
for _, t in ipairs(item.types) do
    if t == "Meeting" then has_meeting = true end
end
```

---

## Writing safe rules

### What you can use

The sandbox allows the full Lua 5.4 standard library for logic and data processing:

- `string` — pattern matching, formatting, manipulation
- `table` — insert, remove, concat, sort
- `math` — floor, ceil, abs, min, max, random
- `utf8` — Unicode string operations
- `pairs`, `ipairs`, `next`, `select`, `type`, `tostring`, `tonumber`
- `pcall`, `xpcall`, `error`, `assert` — error handling
- `setmetatable`, `getmetatable` — OOP patterns

### What is blocked

The following are **not available** in scripts (accessing them causes a runtime error):

- `io` — no file system access
- `os` — no system calls, no `os.execute`, no `os.time` (use `zealot.now` instead)
- `require`, `dofile`, `loadfile` — no loading external modules
- `package`, `debug` — no module system, no debug introspection

### Execution limits

- **Instruction limit:** ~10 million Lua VM instructions per run. Scripts that loop for too long are interrupted automatically.
- **Wall-clock timeout:** 10 seconds. Even if a script is waiting on API calls, it will be killed after 10 seconds.

Both limits exist to protect the server. In practice, even complex scripts finish in milliseconds.

### Handling errors gracefully

Use `pcall` to catch errors without failing the whole rule:

```lua
local ok, err = pcall(function()
    local items = zealot.items.find_by_type("Task")
    for _, item in ipairs(items) do
        zealot.items.set_attribute(item.id, "Processed", "true")
    end
end)

if not ok then
    zealot.notify("Error: " .. tostring(err))
end
```

### Event recursion

An event rule that modifies items does **not** trigger further event rules. Zealot automatically suppresses recursive event firing — a rule that calls `zealot.items.update(...)` will not cause another `on_item_update` rule to fire. This prevents infinite loops.

---

## Building content with ZealotScript

Lua scripts can write content to items using `zealot.items.update`. Content is stored as ZealotScript — the same markdown format the built-in editor uses.

### Supported formatting

```lua
-- Headings
"# Heading 1\n"
"## Heading 2\n"

-- Lists
"- First item\n"
"- Second item\n"

"1. First\n"
"2. Second\n"

-- Bold and italic
"**bold text**"
"_italic text_"

-- Tables (GitHub Flavored Markdown)
"| Column A | Column B |\n"
"|----------|----------|\n"
"| value 1  | value 2  |\n"

-- Code blocks
"```\ncode here\n```\n"

-- Admonitions (Zealot extension)
"!!! note\n    Body text here\n"
```

### Building multi-line content

Use a table of lines and `table.concat`:

```lua
local lines = {
    "# Report",
    "",
    "| Item | Status |",
    "|------|--------|",
}
for _, item in ipairs(items) do
    local status = item.attributes["Status"] or "—"
    table.insert(lines, "| " .. item.title .. " | " .. status .. " |")
end
local content = table.concat(lines, "\n")
```

### Updating an existing item

```lua
local report = zealot.items.get_by_title("Weekly Report")
if report then
    zealot.items.update(report.id, { content = content })
else
    zealot.items.create("Weekly Report", content, { types = { "Report" } })
end
```

The item is then viewable and editable in the ZealotScript editor like any other item. The editor will parse the markdown and display it with full formatting.

---

## Cookbook

### Auto-tag items by title keyword

Automatically assigns a type and sets a default status when an item is created with certain words in its title.

**Trigger:** `on_item_create`

```lua
local item = zealot.event.item
local lower = string.lower(item.title)

if string.find(lower, "meeting") then
    zealot.items.assign_type(item.id, "Meeting")
    zealot.items.set_attribute(item.id, "Status", "Draft")
    zealot.notify("Tagged as Meeting: " .. item.title)
elseif string.find(lower, "review") then
    zealot.items.assign_type(item.id, "Review")
end
```

---

### Stamp a completion date automatically

When Status is set to "Complete", record the date it happened.

**Trigger:** `on_attribute_set` — attribute key: `Status`

```lua
local item = zealot.event.item
if item.attributes["Status"] == "Complete" then
    zealot.items.set_attribute(item.id, "Completed At", zealot.date)
end
```

---

### Apply a content template on type assignment

Pre-fills an empty item with boilerplate when a type is assigned.

**Trigger:** `on_type_assign` — type name: `Meeting`

```lua
local item = zealot.event.item
if not item.content or item.content == "" then
    local template = table.concat({
        "# Agenda",
        "",
        "- ",
        "",
        "# Notes",
        "",
        "- ",
        "",
        "# Action Items",
        "",
        "- ",
    }, "\n")
    zealot.items.update(item.id, { content = template })
    zealot.notify("Applied Meeting template to: " .. item.title)
end
```

---

### Daily agenda item

Every morning, update (or create) a "Today" item with everything due that day.

**Trigger:** Cron — `0 7 * * *` (7 AM daily)

```lua
local items = zealot.items.filter({
    { key = "Date", op = "eq", value = zealot.date }
})

local lines = { "# Today — " .. zealot.date, "" }

if #items == 0 then
    table.insert(lines, "_Nothing scheduled for today._")
else
    for _, item in ipairs(items) do
        local status = item.attributes["Status"] or "—"
        table.insert(lines, "- **[" .. status .. "]** " .. item.title)
    end
end

local content = table.concat(lines, "\n")
local today = zealot.items.get_by_title("Today")
if today then
    zealot.items.update(today.id, { content = content })
else
    zealot.items.create("Today", content)
end
zealot.notify("Updated Today with " .. #items .. " items.")
```

---

### Remind about items due tomorrow

Add a comment to every item due tomorrow that isn't already complete.

**Trigger:** Cron — `0 8 * * *` (8 AM daily)

```lua
-- Build tomorrow's date by incrementing the last two digits
-- zealot.date is YYYY-MM-DD; we rely on the filter doing string comparison
-- For a proper date calculation, use attributes set to tomorrow's value in your workflow,
-- or use this approximation which works for same-month dates:
local year  = string.sub(zealot.date, 1, 4)
local month = string.sub(zealot.date, 6, 7)
local day   = tonumber(string.sub(zealot.date, 9, 10)) + 1
local tomorrow = string.format("%s-%s-%02d", year, month, day)

local items = zealot.items.filter({
    { key = "Date",   op = "eq", value = tomorrow },
    { key = "Status", op = "ne", value = "Complete" },
})

for _, item in ipairs(items) do
    zealot.comments.add(item.id, "Reminder: this item is due tomorrow (" .. tomorrow .. ").")
    zealot.notify("Reminded: " .. item.title)
end

zealot.notify("Sent " .. #items .. " reminders.")
```

---

### Weekly review report

Every Sunday evening, create a new review item summarising work completed this week.

**Trigger:** Cron — `0 20 * * 0` (Sunday at 8 PM)

```lua
local completed = zealot.items.filter({
    { key = "Status", op = "eq", value = "Complete" }
})

local lines = {
    "# Week Review — " .. zealot.date,
    "",
    "## Completed",
    "",
}

for _, item in ipairs(completed) do
    local completed_at = item.attributes["Completed At"] or "—"
    table.insert(lines, "- " .. item.title .. " _(completed " .. completed_at .. ")_")
end

if #completed == 0 then
    table.insert(lines, "_Nothing marked complete this week._")
end

zealot.items.create("Week Review " .. zealot.date, table.concat(lines, "\n"), {
    types = { "Review" }
})
zealot.notify("Created week review with " .. #completed .. " completed items.")
```

---

### Goal progress rollup

Daily, compute what % of each Goal's child items are complete and write it back as an attribute.

**Trigger:** Cron — `0 0 * * *` (midnight)

```lua
local goals = zealot.items.find_by_type("Goal")

for _, goal in ipairs(goals) do
    local children = zealot.items.filter({
        { key = "Parent", op = "eq", value = goal.id }
    })
    local total = #children
    if total > 0 then
        local done = 0
        for _, child in ipairs(children) do
            if child.attributes["Status"] == "Complete" then
                done = done + 1
            end
        end
        local pct = math.floor((done / total) * 100)
        zealot.items.set_attribute(goal.id, "Progress", pct)
        zealot.notify(goal.title .. ": " .. done .. "/" .. total .. " (" .. pct .. "%)")
    end
end
```

---

### Flag stale in-progress items

Every Monday, add a comment to items that are still "In Progress" past their due date.

**Trigger:** Cron — `0 9 * * 1` (Monday at 9 AM)

```lua
local items = zealot.items.filter({
    { key = "Status", op = "eq", value = "In Progress" }
})

local flagged = 0
for _, item in ipairs(items) do
    local due = item.attributes["Date"]
    -- ISO dates compare correctly as strings (YYYY-MM-DD < YYYY-MM-DD)
    if due and due < zealot.date then
        zealot.comments.add(item.id,
            "This item has been In Progress past its due date (" .. due .. "). Still relevant?")
        flagged = flagged + 1
    end
end

zealot.notify("Flagged " .. flagged .. " stale items.")
```

---

### Data integrity check

Every week, find items of a given type that are missing required attributes.

**Trigger:** Cron — `0 9 * * 1` (Monday at 9 AM)

```lua
local projects = zealot.items.find_by_type("Project")
local issues = 0

for _, project in ipairs(projects) do
    if not project.attributes["Status"] then
        zealot.comments.add(project.id, "Data check: missing Status attribute.")
        issues = issues + 1
    end
    if not project.attributes["Date"] then
        zealot.comments.add(project.id, "Data check: missing due Date attribute.")
        issues = issues + 1
    end
end

zealot.notify("Found " .. issues .. " data issues across " .. #projects .. " projects.")
```

---

### One-off bulk rename (manual migration)

Rename an attribute key across all items in one run. Disable the rule after running.

**Trigger:** Manual

```lua
-- Rename attribute "Prio" → "Priority" on all items that have it
local items = zealot.items.filter({
    { key = "Prio", op = "ne", value = "" }
})

local count = 0
for _, item in ipairs(items) do
    local val = item.attributes["Prio"]
    if val then
        zealot.items.set_attribute(item.id, "Priority", val)
        count = count + 1
    end
end

zealot.notify("Migrated 'Prio' → 'Priority' on " .. count .. " items.")
zealot.notify("Remember to disable this rule after confirming the results.")
```

---

### Habit streak counter

Each night, increment a "Streak" counter on all Habit items. (Reset logic can be added via a separate rule or manual step.)

**Trigger:** Cron — `0 23 * * *` (11 PM daily)

```lua
local habits = zealot.items.find_by_type("Habit")

for _, habit in ipairs(habits) do
    local current = tonumber(habit.attributes["Streak"]) or 0
    local new_streak = current + 1
    zealot.items.set_attribute(habit.id, "Streak", new_streak)
    zealot.notify(habit.title .. ": streak now " .. new_streak)
end
```

---

### Task status report (generated document)

Generate a formatted markdown table of all tasks and update a report item.

**Trigger:** Manual or Cron — `0 9 * * 1` (Monday morning)

```lua
local tasks = zealot.items.find_by_type("Task")

local lines = {
    "# Task Status Report — " .. zealot.date,
    "",
    "| Task | Status | Due Date | Priority |",
    "|------|--------|----------|----------|",
}

for _, task in ipairs(tasks) do
    local status   = task.attributes["Status"]   or "—"
    local due      = task.attributes["Date"]      or "—"
    local priority = task.attributes["Priority"]  or "—"
    table.insert(lines,
        "| " .. task.title .. " | " .. status .. " | " .. due .. " | " .. priority .. " |")
end

local content = table.concat(lines, "\n")
local report = zealot.items.get_by_title("Task Status Report")
if report then
    zealot.items.update(report.id, { content = content })
    zealot.notify("Updated Task Status Report.")
else
    zealot.items.create("Task Status Report", content, { types = { "Report" } })
    zealot.notify("Created Task Status Report.")
end
```

---

### Auto-link wiki references

When an item is created containing `[[Title]]` syntax, find the referenced items and log them for manual linking.

**Trigger:** `on_item_create`

```lua
local item = zealot.event.item
local found_any = false

for ref in string.gmatch(item.content, "%[%[(.-)%]%]") do
    local linked = zealot.items.get_by_title(ref)
    if linked then
        zealot.notify("Found reference: [[" .. ref .. "]] → " .. linked.id)
        found_any = true
    else
        zealot.notify("Unresolved reference: [[" .. ref .. "]]")
    end
end

if not found_any then
    -- No wiki links in this item — nothing to do
end
```

---

*For the technical architecture of the rules engine, see [docs/ai-tasks/rules-engine/architecture.md](ai-tasks/rules-engine/architecture.md).*
