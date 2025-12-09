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

class MonthlyPlannerScreen extends BaseElement<DateTime> {
    async render() {
        let date = this.data!;
        this.classList.add('center')
        this.innerHTML = 
        `<h1>${date.monthLong} - ${date.year}</h1>
        <div name="items" style="display: grid; grid-gap: 10px"></div>`;

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
    }
}

customElements.define('monthly-planner-screen', MonthlyPlannerScreen)

export default MonthlyPlannerScreen;