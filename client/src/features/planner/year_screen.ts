import ButtonGroup, {ButtonDef} from "../../shared/button_group";
import { router } from "../router/router";
import { DateTime } from "luxon";
import BaseElement from "../../shared/base_element";
import API from "../../api/api";
import type ItemListView from "../item/item_list_view";
import { HomeIcon, BackIcon, ForwardIcon, DocIcon } from "../../assets/asset_map";

class AnnualPlannerScreen extends BaseElement<DateTime> {
    async render() {
        let date = this.data!;
        this.classList.add('center')
        this.innerHTML = 
        `<h1>${date.year} Planner</h1>
        <item-list-view></item-list-view>
        <!--<add-item-scoped></add-item-scoped>-->
        <div name="items" style="display: grid; grid-gap: 10px"></div>`;

        this.prepend(new ButtonGroup().init([
            new ButtonDef(HomeIcon, "This Year", () => {
                let this_year = DateTime.now();
                let str = this_year.year;
                router.navigate(`/planner/annual/${str}`)
            }),
            new ButtonDef(BackIcon, "Last Year", () => {
                let last_year = date.minus({year: 1});
                let str = last_year.year;
                router.navigate(`/planner/annual/${str}`)
            }),
            new ButtonDef(ForwardIcon, "Next Year", () => {
                let next_year = date.plus({year: 1});
                let str = next_year.year;
                router.navigate(`/planner/annual/${str}`)
            }),
            new ButtonDef(DocIcon, 'Create Note', () => {
                router.navigate(`/item/${date.toFormat('yyyy')}`)
            })
        ]));  

        let items_view = this.querySelector('item-list-view')! as ItemListView;
        try {
            items_view
                .enable_add_item(
                    {
                        Year: date.year,
                        Status: "To Do",
                        Priority: 3,
                        Icon: ''
                    },
                    async () => {
                        items_view.only_render_items = true;
                        items_view.data = await API.planner.get_items_for_year(date!)
                    }
                )
                .init(await API.planner.get_items_for_year(date!))
        } catch (e) {
            console.error(e)
        }
    } 
}

customElements.define('annual-planner-screen', AnnualPlannerScreen)

export default AnnualPlannerScreen;