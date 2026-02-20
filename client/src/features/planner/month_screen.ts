import { DateTime } from "luxon";
import BaseElement from "../../shared/base_element";
import { HomeIcon, BackIcon, ForwardIcon, SunIcon, DocIcon } from "../../assets/asset_map";

import ButtonGroup, {ButtonDef} from "../../shared/button_group";
import { router } from "../router/router";
import type AddItemScoped from "../item/add_item_scope";
import API from "../../api/api";
import type ItemListView from "../item/item_list_view";

class MonthlyPlannerScreen extends BaseElement<DateTime> {
    async render() {
        let date = this.data!;
        this.classList.add('center')
        this.innerHTML = 
        `<h1>${date.monthLong} - ${date.year}</h1>
        <item-list-view></item-list-view>
        <div name="items" style="display: grid; grid-gap: 10px"></div>`;

        this.prepend(new ButtonGroup().init([
            new ButtonDef(HomeIcon, "This Month", () => {
                let this_month = DateTime.now();
                let str = this_month.toFormat(`yyyy-MM`);
                router.navigate(`/planner/monthly/${str}`)
            }),
            new ButtonDef(BackIcon, "Last Month", () => {
                let last_month = date.minus({month: 1});
                let str = last_month.toFormat(`yyyy-MM`);
                router.navigate(`/planner/monthly/${str}`)
            }),
            new ButtonDef(ForwardIcon, "Next Month", () => {
                let next_month = date.plus({month: 1});
                let str = next_month.toFormat(`yyyy-MM`);
                router.navigate(`/planner/monthly/${str}`)
            }),
            new ButtonDef(SunIcon, `${date.year}`, () => {
                router.navigate(`/planner/annual/${date.year}`)
            }),
            new ButtonDef(DocIcon, 'Create Note', () => {
                router.navigate(`/item/${date.toFormat('MMMM yyyy')}`)
            }),
        ]));  

        let items_view = this.querySelector('item-list-view')! as ItemListView;
        try {
            items_view
                .enable_add_item(
                    {
                        Month: date.month,
                        Year: date.year,
                        Status: "To Do",
                        Priority: 3,
                        Icon: ''
                    },
                    async () => {
                        items_view.only_render_items = true;
                        items_view.data = await API.planner.get_items_for_month(date!)
                    }
                )
                .init(await API.planner.get_items_for_month(date!))
        } catch (e) {
            console.error(e)
        }
    }
}

customElements.define('monthly-planner-screen', MonthlyPlannerScreen)

export default MonthlyPlannerScreen;