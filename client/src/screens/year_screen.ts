
import HomeIcon from "../assets/icon/home.svg";
import PreviousIcon from "../assets/icon/back.svg";
import NextIcon from "../assets/icon/forward.svg";
import DocIcon from "../assets/icon/doc.svg";
import ButtonGroup, { ButtonDef } from "../components/common/button_group";
import { router } from "../core/router";
import { DateTime } from "luxon";
import BaseElement from "../components/common/base_element";
import type AddItemScoped from "../components/add_item_scope";
import API from "../api/api";
import PlanView from "../components/item_view";
import type ItemListView from "../components/item_list_view";


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
            new ButtonDef(PreviousIcon, "Last Year", () => {
                let last_year = date.minus({year: 1});
                let str = last_year.year;
                router.navigate(`/planner/annual/${str}`)
            }),
            new ButtonDef(NextIcon, "Next Year", () => {
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
                    async () => {items_view.data = await API.planner.get_items_for_year(date!)}
                )
                .init(await API.planner.get_items_for_year(date!))
        } catch (e) {
            console.error(e)
        }
    } 
}

customElements.define('annual-planner-screen', AnnualPlannerScreen)

export default AnnualPlannerScreen;