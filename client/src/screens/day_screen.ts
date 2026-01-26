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
}

customElements.define('daily-planner-screen', DailyPlannerScreen)

export default DailyPlannerScreen;
