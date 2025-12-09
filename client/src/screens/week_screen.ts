import { DateTime } from "luxon";
import BaseElement from "../components/common/base_element";
import ButtonGroup, { ButtonDef } from "../components/common/button_group";
import HomeIcon from "../assets/icon/home.svg";
import PreviousIcon from "../assets/icon/back.svg";
import NextIcon from "../assets/icon/forward.svg";
import MonthIcon from "../assets/icon/moon.svg";
import YearIcon from "../assets/icon/sun.svg";
import DocIcon from "../assets/icon/doc.svg";
import { router } from "../core/router";
import type AddItemScoped from "../components/add_item_scope";
import API from "../api/api";
import PlanView from "../components/plan_view";

class WeeklyPlannerScreen extends BaseElement<DateTime> {
    async render() {
        let date = this.data!;
        this.classList.add('center')
        this.innerHTML = 
        `<h1>Week ${date.weekNumber} - ${date.year}</h1>
        <add-item-scoped></add-item-scoped>
        <div name="items" style="display: grid; grid-gap: 10px"></div>`;

        let add_item = this.querySelector('add-item-scoped')! as AddItemScoped;
        add_item.init({
            Week: date.toISOWeekDate()?.substring(0, 8),
            Status: "To Do",
            Priority: 3,
            Icon: ''
        })
        add_item.listen_on_submit(() => {this.render()});

        this.prepend(new ButtonGroup().init([
            new ButtonDef(HomeIcon, "This Week", () => {
                let this_week = DateTime.now();
                let str = this_week.toISOWeekDate()?.substring(0, 8);
                router.navigate(`/planner/weekly/${str}`)
            }),
            new ButtonDef(PreviousIcon, "Last Week", () => {
                let last_week = date.minus({week: 1});
                let str = last_week.toISOWeekDate()?.substring(0, 8);
                router.navigate(`/planner/weekly/${str}`)
            }),
            new ButtonDef(NextIcon, "Last Week", () => {
                let next_week = date.plus({week: 1});
                let str = next_week.toISOWeekDate()?.substring(0, 8);
                router.navigate(`/planner/weekly/${str}`)
            }),
            new ButtonDef(MonthIcon, `${date.monthLong} ${date.year}`, () => {
                let month_str = date.toFormat(`yyyy-MM`)
                router.navigate(`/planner/monthly/${month_str}`);
            }),
            new ButtonDef(YearIcon, `${date.year}`, () => {
                router.navigate(`/planner/annual/${date.year}`)
            }),
            new ButtonDef(DocIcon, 'Create Note', () => {
                router.navigate(`/item/${date.toISOWeekDate()!.substring(0, 8)}`)
            })
        ]));
        
        let items_container = this.querySelector('[name="items"]')! as HTMLElement;
        try {
            let items = await API.planner.get_items_for_week(date);
            items.forEach(item => {
                let view = new PlanView();
                view.item = item;
                items_container.appendChild(view)
            })
            if (items.length == 0) {
                items_container.innerHTML = "No items scheduled for this week"
            }

        } catch (e) {
            console.error(e)
            items_container.innerHTML = "<div class='error'>Error getting items</div>"
        }
    }
}

customElements.define('weekly-planner-screen', WeeklyPlannerScreen)

export default WeeklyPlannerScreen;