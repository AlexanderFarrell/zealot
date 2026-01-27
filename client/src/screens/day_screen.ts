import { DateTime } from "luxon";
import API from "../api/api";
import PlanView from "../components/item_view";
import { router } from "../core/router";
import HomeIcon from "../assets/icon/home.svg";
import PreviousIcon from "../assets/icon/back.svg";
import NextIcon from "../assets/icon/forward.svg";
import WeekIcon from "../assets/icon/week.svg";
import MonthIcon from "../assets/icon/moon.svg";
import YearIcon from "../assets/icon/sun.svg";
import DocIcon from "../assets/icon/doc.svg";
import BaseElement from "../components/common/base_element";
import ButtonGroup, { ButtonDef } from "../components/common/button_group";
import AddItemScoped from "../components/add_item_scope";
import type ItemListView from "../components/item_list_view";
import DragUtil from "../core/drag_helper";

class DailyPlannerScreen extends BaseElement<DateTime> {
    async render() {
        let date = this.data!;
        this.classList.add('center')
        this.innerHTML = `
        <h1>${date.toFormat(`EEEE - d MMMM yyyy`)}</h1>
        <item-list-view></item-list-view>
        <div class="repeat-planner">
            <h2>Repeats</h2>
            <div name="repeat_list" style="display: grid; gap: 8px;"></div>
        </div>
        <div class="tracker-planner">
            <div style="display: grid; grid-template-columns: 1fr auto">
                <h2>Tracker</h2>
                <button type="button" name="tracker_toggle">Add</button>
            </div>
            <form name="tracker_form" style="display: none; gap: 8px;">
                <div style="display: grid; grid-template-columns: 1fr 8em 6em 2fr 6em; gap: 8px; align-items: center;">
                    <select name="tracker_item"></select>
                    <input type="time" name="tracker_time">
                    <input type="number" name="tracker_level" min="1" max="10" value="3">
                    <input type="text" name="tracker_comment" placeholder="Comment">
                    <button type="submit">Log</button>
                </div>
            </form>
            <div name="tracker_list" style="display: grid; gap: 8px;"></div>
        </div>
        <div name="items" style="display: grid; grid-gap: 10px"></div>`

        this.prepend(new ButtonGroup().init([
            new ButtonDef(HomeIcon, "Today", () => {
                let today = DateTime.now()
                router.navigate(`/planner/daily/${today.toISODate()}`);
            }),
            new ButtonDef(PreviousIcon, "Previous Day", () => {
                let previous = date.minus({days: 1})
                router.navigate(`/planner/daily/${previous.toISODate()}`)
            }),
            new ButtonDef(NextIcon, "Next Day", () => {
                let next = date.plus({days: 1});
                router.navigate(`/planner/daily/${next.toISODate()}`)
            }),
            new ButtonDef(WeekIcon, `Week ${date.weekNumber} - ${date.year}`, () => {
                let week = date.toISOWeekDate()?.substring(0, 8);
                router.navigate(`/planner/weekly/${week}`)
            }),
            new ButtonDef(MonthIcon, `${date.monthLong} ${date.year}`, () => {
                let month_str = date.toFormat(`yyyy-MM`)
                router.navigate(`/planner/monthly/${month_str}`);
            }),
            new ButtonDef(YearIcon, `${date.year}`, () => {
                router.navigate(`/planner/annual/${date.year}`)
            }),
            new ButtonDef(DocIcon, 'Create Note', () => {
                router.navigate(`/item/${date!.toISODate()}`)
            })
        ]))

        let items_view = this.querySelector('item-list-view')! as ItemListView;
        try {
            items_view
                .enable_add_item(
                    {
                        Date: date.toISODate(),
                        Status: "To Do",
                        Priority: 3,
                        Icon: ''
                    },
                    async () => {
                        items_view.only_render_items = true;
                        items_view.data = await API.planner.get_items_on_day(date!)
                    }
                )
                .init(await API.planner.get_items_on_day(date!))
        } catch (e) {
            console.error(e)
        }

        DragUtil.setup_drop(this.querySelector('[title="Previous Day"]')!, 
            {"Date": this.data!.minus({days: 1}).toISODate()});
        DragUtil.setup_drop(this.querySelector('[title="Next Day"]')!, 
            {"Date": this.data!.plus({days: 1}).toISODate()});

        await this.render_repeats(date);
        await this.render_tracker(date);
    }

    private async render_repeats(date: DateTime) {
        const container = this.querySelector('[name="repeat_list"]') as HTMLDivElement;
        if (!container) {
            return;
        }
        container.innerHTML = "";

        let repeats = [];
        try {
            repeats = await API.repeat.get_for_day(date);
        } catch (e) {
            console.error(e);
            container.innerText = "Error loading repeats.";
            return;
        }

        if (!repeats || repeats.length == 0) {
            container.innerText = "No repeats scheduled.";
            return;
        }

        const order = ["Morning", "Afternoon", "Evening", "Anytime"];
        const buckets: Record<string, typeof repeats> = {
            Morning: [],
            Afternoon: [],
            Evening: [],
            Anytime: []
        };

        repeats.forEach((entry) => {
            const raw = entry.item.attributes?.["Time of Day"];
            const label = typeof raw === "string" ? raw : "Anytime";
            if (label in buckets) {
                buckets[label].push(entry);
            } else {
                buckets.Anytime.push(entry);
            }
        });

        order.forEach((label) => {
            const group = buckets[label];
            if (!group || group.length == 0) {
                return;
            }

            const header = document.createElement("div");
            header.classList.add("repeat-group");
            header.innerText = label;
            container.appendChild(header);

            group.forEach((entry) => {
                const row = document.createElement("div");
                row.classList.add("repeat-row");
                row.style.display = "grid";
                row.style.gridTemplateColumns = "1.5em 1fr 2fr 8em";
                row.style.alignItems = "center";
                row.style.gap = "8px";

                const checkbox = document.createElement("input");
                checkbox.type = "checkbox";
                checkbox.checked = entry.status !== "Not Completed";

                const title = document.createElement("div");
                title.classList.add("repeat-title");
                title.innerText = (entry.item.attributes!['Icon'] || "") + " " + entry.item.title;
                title.style.cursor = "pointer";
                title.addEventListener("click", () => {
                    router.navigate(`/item_id/${entry.item.item_id}`);
                });

                const comment = document.createElement("input");
                comment.type = "text";
                comment.placeholder = "Comment";
                comment.value = entry.comment || "";

                const statusLabel = document.createElement("div");
                statusLabel.classList.add("repeat-status");
                statusLabel.innerText = entry.status;

                const save = async () => {
                    const status = checkbox.checked ? "Complete" : "Not Completed";
                    const commentText = comment.value || "";
                    statusLabel.innerText = status;
                    try {
                        await API.repeat.set_status(entry.item.item_id, date, status, commentText);
                    } catch (e) {
                        console.error(e);
                    }
                };

                checkbox.addEventListener("change", save);
                comment.addEventListener("blur", save);

                row.appendChild(checkbox);
                row.appendChild(title);
                row.appendChild(comment);
                row.appendChild(statusLabel);
                container.appendChild(row);
            });
        });
    }

    private async render_tracker(date: DateTime) {
        const list = this.querySelector('[name="tracker_list"]') as HTMLDivElement;
        const form = this.querySelector('[name="tracker_form"]') as HTMLFormElement;
        const itemSelect = this.querySelector('[name="tracker_item"]') as HTMLSelectElement;
        const timeInput = this.querySelector('[name="tracker_time"]') as HTMLInputElement;
        const levelInput = this.querySelector('[name="tracker_level"]') as HTMLInputElement;
        const commentInput = this.querySelector('[name="tracker_comment"]') as HTMLInputElement;
        const toggle = this.querySelector('[name="tracker_toggle"]') as HTMLButtonElement;

        if (!list || !form || !itemSelect || !timeInput || !levelInput || !commentInput || !toggle) {
            return;
        }

        if (toggle.dataset.bound !== "1") {
            toggle.dataset.bound = "1";
            toggle.addEventListener("click", () => {
                const isHidden = form.style.display === "none";
                form.style.display = isHidden ? "grid" : "none";
                toggle.innerText = isHidden ? "Close" : "Add";
            });
        }

        list.innerHTML = "";
        itemSelect.innerHTML = "";

        try {
            const trackerItems = await API.item.get_by_type("Tracker");
            trackerItems.forEach((it) => {
                const opt = document.createElement("option");
                opt.value = String(it.item_id);
                opt.innerText = it.title;
                itemSelect.appendChild(opt);
            });
        } catch (e) {
            console.error(e);
        }

        if (!timeInput.value) {
            const now = DateTime.now();
            timeInput.value = now.toFormat("HH:mm");
        }

        const loadEntries = async () => {
            list.innerHTML = "";
            let entries = [];
            try {
                entries = await API.tracker.get_for_day(date);
            } catch (e) {
                console.error(e);
                list.innerText = "Error loading tracker entries.";
                return;
            }

            if (!entries || entries.length == 0) {
                list.innerText = "No tracker entries.";
                return;
            }

            entries.forEach((entry) => {
                const row = document.createElement("div");
                row.style.display = "grid";
                row.style.gridTemplateColumns = "6em 1fr 4em 2fr 4em";
                row.style.alignItems = "center";
                row.style.gap = "8px";

                const time = DateTime.fromISO(entry.timestamp).toFormat("HH:mm");
                const timeLabel = document.createElement("div");
                timeLabel.innerText = time;

                const title = document.createElement("div");
                title.innerText = entry.item.title;
                title.style.cursor = "pointer";
                title.addEventListener("click", () => {
                    router.navigate(`/item/${entry.item.item_id}`);
                });

                const level = document.createElement("div");
                level.innerText = String(entry.level);

                const comment = document.createElement("div");
                comment.innerText = entry.comment || "";

                const del = document.createElement("button");
                del.innerText = "Delete";
                del.addEventListener("click", async () => {
                    try {
                        await API.tracker.delete_entry(entry.tracker_id);
                        await loadEntries();
                    } catch (e) {
                        console.error(e);
                    }
                });

                row.appendChild(timeLabel);
                row.appendChild(title);
                row.appendChild(level);
                row.appendChild(comment);
                row.appendChild(del);
                list.appendChild(row);
            });
        };

        if (form.dataset.bound !== "1") {
            form.dataset.bound = "1";
            form.addEventListener("submit", async (e) => {
                e.preventDefault();
                if (!itemSelect.value) {
                    return;
                }

                const [hour, minute] = (timeInput.value || "00:00").split(":").map(Number);
                const timestamp = date.set({
                    hour: Number.isFinite(hour) ? hour : 0,
                    minute: Number.isFinite(minute) ? minute : 0,
                    second: 0,
                    millisecond: 0
                });

                const level = Math.max(1, Math.min(10, parseInt(levelInput.value || "3", 10)));
                const comment = commentInput.value || "";

                try {
                    await API.tracker.add_entry(parseInt(itemSelect.value, 10), timestamp, level, comment);
                    commentInput.value = "";
                    await loadEntries();
                } catch (e) {
                    console.error(e);
                }
            });
        }

        await loadEntries();
    }
}

customElements.define('daily-planner-screen', DailyPlannerScreen)

export default DailyPlannerScreen;
