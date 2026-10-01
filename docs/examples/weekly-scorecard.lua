-- Weekly scorecard: runs entirely inside Zealot, no agent needed.
-- Week = Monday..Sunday in the user's time zone. Pass any date in the week
-- to score a past week, e.g. local anchor = "2026-09-30".
local anchor = nil
local week_start = zealot.week_start(anchor)
local week_end = zealot.date_add(week_start, 6)

-- Shipped: Tickets whose Status is Complete and whose Completed At falls in the week.
local shipped, craft_minutes, receipts = 0, 0, 0
for _, ticket in ipairs(zealot.items.find_by_type("Ticket")) do
    local attrs = ticket.attributes
    if attrs["Status"] == "Complete" and attrs["Completed At"] then
        local day = zealot.local_date(attrs["Completed At"]) or string.sub(attrs["Completed At"], 1, 10)
        if day >= week_start and day <= week_end then
            shipped = shipped + 1
        end
    end
end

-- Evidence: comments written each day of the week, by local day.
local comments = 0
for offset = 0, 6 do
    local day = zealot.date_add(week_start, offset)
    for _, c in ipairs(zealot.comments.for_date(day)) do
        if zealot.local_date(c.timestamp) == day then
            comments = comments + 1
        end
    end
end

-- Craft hours: time blocks on items titled "Craft ...".
for _, block in ipairs(zealot.time_blocks.for_range(week_start, week_end)) do
    if string.find(block.title, "^Craft") then
        craft_minutes = craft_minutes + block.minutes
        if block.note ~= "" then receipts = receipts + 1 end
    end
end

-- Habits: completion across the week (days after today are not counted).
local habits = zealot.habits.stats(week_start, week_end)

zealot.notify("Scorecard " .. week_start .. " to " .. week_end .. " (" .. zealot.timezone .. ")")
zealot.notify("Shipped: " .. shipped)
zealot.notify("Evidence comments: " .. comments)
zealot.notify(string.format("Craft: %.1f h (%d blocks with receipts)", craft_minutes / 60, receipts))
for _, h in ipairs(habits.habits) do
    zealot.notify(string.format("Habit %s: %d complete, streak %d", h.title, h.counts.complete, h.current_streak))
end
