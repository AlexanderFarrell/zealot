
import HomeIcon from "../assets/icon/home.svg";
import PreviousIcon from "../assets/icon/back.svg";
import NextIcon from "../assets/icon/forward.svg";
import ButtonGroup, { ButtonDef } from "../components/common/button_group";
import { router } from "../core/router";
import { DateTime } from "luxon";
import BaseElement from "../components/common/base_element";


class AnnualPlannerScreen extends BaseElement<DateTime> {
    async render() {
        let date = this.data!;
        this.classList.add('center')
        this.innerHTML = 
        `<h1>${date.year} Planner</h1>
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
        ]));  
    } 
}

customElements.define('annual-planner-screen', AnnualPlannerScreen)

export default AnnualPlannerScreen;