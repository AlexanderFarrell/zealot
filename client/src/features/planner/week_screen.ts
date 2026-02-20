import { DateTime } from "luxon";
import BaseElement from "../../shared/base_element";
import ButtonGroup, {ButtonDef} from "../../shared/button_group";
import { HomeIcon, BackIcon, ForwardIcon, MoonIcon, SunIcon, DocIcon } from "../../assets/asset_map";

import { router } from "../router/router";
import type AddItemScoped from "../item/add_item_scope";
import API from "../../api/api";
import PlanView from "../item/item_view";
import type ItemListView from "../item/item_list_view";

class WeeklyPlannerScreen extends BaseElement<DateTime> {
    async render() {
        let date = this.data!;
        this.classList.add('center')
        this.innerHTML = 
        `<h1>Week ${date.weekNumber} - ${date.year}</h1>
        <item-list-view></item-list-view>
        <div name="items" style="display: grid; grid-gap: 10px"></div>`;

        this.prepend(new ButtonGroup().init([
            new ButtonDef(HomeIcon, "This Week", () => {
                let this_week = DateTime.now();
                let str = this_week.toISOWeekDate()?.substring(0, 8);
                router.navigate(`/planner/weekly/${str}`)
            }),
            new ButtonDef(BackIcon, "Last Week", () => {
                let last_week = date.minus({week: 1});
                let str = last_week.toISOWeekDate()?.substring(0, 8);
                router.navigate(`/planner/weekly/${str}`)
            }),
            new ButtonDef(ForwardIcon, "Last Week", () => {
                let next_week = date.plus({week: 1});
                let str = next_week.toISOWeekDate()?.substring(0, 8);
                router.navigate(`/planner/weekly/${str}`)
            }),
            new ButtonDef(MoonIcon, `${date.monthLong} ${date.year}`, () => {
                let month_str = date.toFormat(`yyyy-MM`)
                router.navigate(`/planner/monthly/${month_str}`);
            }),
            new ButtonDef(SunIcon, `${date.year}`, () => {
                router.navigate(`/planner/annual/${date.year}`)
            }),
            new ButtonDef(DocIcon, 'Create Note', () => {
                router.navigate(`/item/${date.toISOWeekDate()!.substring(0, 8)}`)
            })
        ]));
        
        let items_view = this.querySelector('item-list-view')! as ItemListView;
        try {
            items_view
                .enable_add_item(
                    {
                        Week: date.toISOWeekDate()?.substring(0, 8),
                        Status: "To Do",
                        Priority: 3,
                        Icon: ''
                    },
                    async () => {
                        items_view.only_render_items = true;
                        items_view.data = await API.planner.get_items_for_week(date!)
                    }
                )
                .init(await API.planner.get_items_for_week(date!))
        } catch (e) {
            console.error(e)
        }
    }
}

customElements.define('weekly-planner-screen', WeeklyPlannerScreen)

export default WeeklyPlannerScreen;