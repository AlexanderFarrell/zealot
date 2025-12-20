import { DateTime } from "luxon";
import BaseElement from "../components/common/base_element";
import HomeIcon from "../assets/icon/home.svg";
import PreviousIcon from "../assets/icon/back.svg";
import NextIcon from "../assets/icon/forward.svg";
import MonthIcon from "../assets/icon/moon.svg";
import YearIcon from "../assets/icon/sun.svg";
import DocIcon from "../assets/icon/doc.svg";
import ButtonGroup, { ButtonDef } from "../components/common/button_group";
import { router } from "../core/router";
import type AddItemScoped from "../components/add_item_scope";
import API from "../api/api";
import PlanView from "../components/plan_view";

class MonthlyPlannerScreen extends BaseElement<DateTime> {
    async render() {
        let date = this.data!;
        this.classList.add('center')
        this.innerHTML = 
        `<h1>${date.monthLong} - ${date.year}</h1>
        <add-item-scoped></add-item-scoped>
        <div name="items" style="display: grid; grid-gap: 10px"></div>`;

        let add_item = this.querySelector('add-item-scoped')! as AddItemScoped;
        add_item.init({
            Month: date.month,
            Year: date.year,
            Status: "To Do",
            Priority: 3,
            Icon: ''
        })
        add_item.listen_on_submit(() => {this.render()});

        this.prepend(new ButtonGroup().init([
            new ButtonDef(HomeIcon, "This Month", () => {
                let this_month = DateTime.now();
                let str = this_month.toFormat(`yyyy-MM`);
                router.navigate(`/planner/monthly/${str}`)
            }),
            new ButtonDef(PreviousIcon, "Last Month", () => {
                let last_month = date.minus({month: 1});
                let str = last_month.toFormat(`yyyy-MM`);
                router.navigate(`/planner/monthly/${str}`)
            }),
            new ButtonDef(NextIcon, "Next Month", () => {
                let next_month = date.plus({month: 1});
                let str = next_month.toFormat(`yyyy-MM`);
                router.navigate(`/planner/monthly/${str}`)
            }),
            new ButtonDef(YearIcon, `${date.year}`, () => {
                router.navigate(`/planner/annual/${date.year}`)
            }),
            new ButtonDef(DocIcon, 'Create Note', () => {
                router.navigate(`/item/${date.toFormat('MMMM yyyy')}`)
            }),
        ]));  
        let items_container = this.querySelector('[name="items"]')! as HTMLElement;
        try {
            let items = await API.planner.get_items_for_month(date!);
            items.forEach(item => {
                let view = new PlanView();
                view.item = item;
                items_container.appendChild(view)
            })
            if (items.length == 0) {
                items_container.innerHTML = "No items scheduled for this month."
            }
        } catch (e) {
            console.error(e)
            items_container.innerHTML = "<div class='error'>Error getting items</div>"
        }
    }
}

customElements.define('monthly-planner-screen', MonthlyPlannerScreen)

export default MonthlyPlannerScreen;