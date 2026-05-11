# Rules Engine — High-Value Use Cases

Each use case below includes the trigger type, a short description, and a Lua script example.

---

## 1. Auto-tag on creation

**Trigger:** `on_item_create`  
**Value:** Eliminates repetitive manual classification of new items.

```lua
local item = zealot.event.item
if string.find(string.lower(item.title), "meeting") then
    zealot.items.assign_type(item.id, "Meeting")
    zealot.items.set_attribute(item.id, "Status", "Draft")
    zealot.notify("Auto-tagged as Meeting: " .. item.title)
end
```

---

## 2. Completion timestamp

**Trigger:** `on_attribute_set` (attribute_key = "Status")  
**Value:** Automatically records when work was completed — no manual "Completed At" entry needed.

```lua
local item = zealot.event.item
if item.attributes["Status"] == "Complete" then
    zealot.items.set_attribute(item.id, "Completed At", zealot.date)
end
```

---

## 3. Due-date reminder comments

**Trigger:** `Cron` — `0 8 * * *` (daily at 8 AM)  
**Value:** Creates a visible reminder on each item due tomorrow.

```lua
local tomorrow = os.date("%Y-%m-%d", os.time() + 86400)  -- use zealot.date + 1 day via string math
local items = zealot.items.filter({
    { key = "Date", op = "eq", value = tomorrow },
    { key = "Status", op = "ne", value = "Complete" },
})
for _, item in ipairs(items) do
    zealot.comments.add(item.id, "Reminder: this item is due tomorrow.")
    zealot.notify("Reminded: " .. item.title)
end
```

---

## 4. Daily digest item

**Trigger:** `Cron` — `0 6 * * *` (daily at 6 AM)  
**Value:** Produces a single "Today" item with an agenda — open it to see your day.

```lua
local items = zealot.items.filter({
    { key = "Date", op = "eq", value = zealot.date }
})
local lines = { "# Today — " .. zealot.date, "" }
for _, item in ipairs(items) do
    local status = item.attributes["Status"] or "—"
    table.insert(lines, "- [" .. status .. "] " .. item.title)
end
local digest = zealot.items.get_by_title("Daily Digest")
if digest then
    zealot.items.update(digest.id, { content = table.concat(lines, "\n") })
else
    zealot.items.create("Daily Digest", table.concat(lines, "\n"))
end
```

---

## 5. Weekly review report

**Trigger:** `Cron` — `0 20 * * 0` (Sunday at 8 PM)  
**Value:** Automatically creates a weekly retrospective item every Sunday.

```lua
local completed = zealot.items.filter({
    { key = "Status", op = "eq", value = "Complete" }
})
local lines = { "# Week Review", "", "## Completed this week", "" }
for _, item in ipairs(completed) do
    if item.attributes["Completed At"] == zealot.date or true then  -- refine with date range
        table.insert(lines, "- " .. item.title)
    end
end
zealot.items.create("Week Review " .. zealot.date, table.concat(lines, "\n"), {
    types = { "Review" }
})
```

---

## 6. Habit streak tracking

**Trigger:** `Cron` — `0 23 * * *` (daily at 11 PM)  
**Value:** Keeps a running streak count on repeat items — motivational and visible in planner.

```lua
-- For each item of type "Habit", count consecutive days completed
-- (requires zealot.items.filter + repeat data; initial implementation uses comment count proxy)
local habits = zealot.items.find_by_type("Habit")
for _, habit in ipairs(habits) do
    local streak = (habit.attributes["Streak"] or 0) + 1
    zealot.items.set_attribute(habit.id, "Streak", streak)
    zealot.notify(habit.title .. " streak: " .. streak)
end
```

---

## 7. Goal progress rollup

**Trigger:** `Cron` — `0 0 * * *` (midnight daily)  
**Value:** Automatically computes % complete for any Project or Goal, visible as an attribute.

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
    end
end
```

---

## 8. Stale item detector

**Trigger:** `Cron` — `0 9 * * 1` (Monday at 9 AM)  
**Value:** Surfaces forgotten in-progress work before the week starts.

```lua
local items = zealot.items.filter({
    { key = "Status", op = "eq", value = "In Progress" }
})
for _, item in ipairs(items) do
    local date = item.attributes["Date"]
    if date and date < zealot.date then  -- simple string compare works for ISO dates
        zealot.comments.add(item.id, "This item has been in progress past its due date. Still relevant?")
        zealot.notify("Flagged stale: " .. item.title)
    end
end
```

---

## 9. Auto-link related items

**Trigger:** `on_item_create`  
**Value:** Builds the knowledge graph automatically — new items are wired to existing ones.

```lua
local item = zealot.event.item
-- Search for existing items whose title appears in the new item's content
local words = {}
for word in string.gmatch(item.content, "%[%[(.-)%]%]") do  -- [[WikiLink]] syntax
    table.insert(words, word)
end
for _, word in ipairs(words) do
    local found = zealot.items.get_by_title(word)
    if found then
        zealot.notify("Auto-linked to: " .. found.title)
        -- Future: zealot.items.link(item.id, found.id, "tag")
    end
end
```

---

## 10. One-off data migration (manual trigger)

**Trigger:** `Manual`  
**Value:** Allows safe, auditable bulk changes to data without writing SQL or a separate script.

```lua
-- Example: rename attribute key "Prio" → "Priority" across all items
local items = zealot.items.filter({
    { key = "Prio", op = "ne", value = "" }
})
local count = 0
for _, item in ipairs(items) do
    local val = item.attributes["Prio"]
    if val then
        zealot.items.set_attribute(item.id, "Priority", val)
        -- Future: zealot.items.remove_attribute(item.id, "Prio")
        count = count + 1
    end
end
zealot.notify("Migrated " .. count .. " items")
```

---

## 11. Data integrity check

**Trigger:** `Cron` — `0 9 * * 1` (weekly)  
**Value:** Catches configuration drift — items missing required attributes.

```lua
local projects = zealot.items.find_by_type("Project")
for _, project in ipairs(projects) do
    if not project.attributes["Status"] then
        zealot.comments.add(project.id, "Data check: Project is missing a Status attribute.")
        zealot.notify("Missing Status: " .. project.title)
    end
    if not project.attributes["Date"] then
        zealot.comments.add(project.id, "Data check: Project has no due date set.")
    end
end
```

---

## 12. Smart item templates on type assignment

**Trigger:** `on_type_assign` (type_name = "Meeting")  
**Value:** Pre-populates boilerplate content when a user assigns a type, saving manual setup.

```lua
local item = zealot.event.item
if item.content == "" or item.content == nil then
    local template = [[
# Agenda

- 

# Notes

- 

# Action Items

- 
]]
    zealot.items.update(item.id, { content = template })
    zealot.notify("Applied Meeting template to: " .. item.title)
end
```

---

## 13. Custom report view (ZealotScript content generation)

**Trigger:** `Manual` or `Cron`  
**Value:** Generates a rich, structured report item viewable in the ZealotScript editor.

```lua
-- Produces a status report item with a markdown table
local items = zealot.items.find_by_type("Task")
local lines = {
    "# Task Status Report — " .. zealot.date,
    "",
    "| Title | Status | Due |",
    "|-------|--------|-----|",
}
for _, item in ipairs(items) do
    local status = item.attributes["Status"] or "—"
    local due = item.attributes["Date"] or "—"
    table.insert(lines, "| " .. item.title .. " | " .. status .. " | " .. due .. " |")
end
local report = zealot.items.get_by_title("Task Status Report")
local content = table.concat(lines, "\n")
if report then
    zealot.items.update(report.id, { content = content })
else
    zealot.items.create("Task Status Report", content, { types = { "Report" } })
end
```

---

## 14. Notification bridge (future)

**Trigger:** Any  
**Value:** When `zealot.notify(msg)` output is captured as `last_output`, a future `NotificationPort` can forward it to email, webhook, or desktop notification based on per-account settings. No Lua changes required — just add the port impl.

---

## 15. ZealotScript live view block (future)

**Trigger:** N/A — rendered at read time  
**Value:** A ZealotScript block type `{{rule:rule-id}}` that renders the rule's `last_output` inline inside any item's content. Useful for embedding computed dashboards inside wiki pages.

This requires:
1. A `last_output` column on the `rule` table (already planned)
2. A new ProseMirror node type `rule_embed` in the ZealotScript schema
3. A backend endpoint `GET /rule/{id}/output` returning the last output text
4. The ZealotScript renderer fetching and injecting it at display time
