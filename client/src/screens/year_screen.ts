
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
import PlanView from "../components/plan_view";


class AnnualPlannerScreen extends BaseElement<DateTime> {
    async render() {
        let date = this.data!;
        this.classList.add('center')
        this.innerHTML = 
        `<h1>${date.year} Planner</h1>
        <add-item-scoped></add-item-scoped>
        <div name="items" style="display: grid; grid-gap: 10px"></div>`;

        let add_item = this.querySelector('add-item-scoped')! as AddItemScoped;
        add_item.init({
            Year: date.year,
            Status: "To Do",
            Priority: 3,
            Icon: ''
        })
        add_item.listen_on_submit(() => {this.render()});

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

        let items_container = this.querySelector('[name="items"]')! as HTMLElement;
        try {
            let items = await API.planner.get_items_for_year(date);
            items.forEach(item => {
                let view = new PlanView();
                view.item = item;
                items_container.appendChild(view)
            })
            if (items.length == 0) {
                items_container.innerHTML = "No items scheduled for this year"
            }

        } catch (e) {
            console.error(e)
            items_container.innerHTML = "<div class='error'>Error getting items</div>"
        }
    } 
}

customElements.define('annual-planner-screen', AnnualPlannerScreen)

export default AnnualPlannerScreen;