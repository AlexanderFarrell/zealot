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

class WeeklyPlannerScreen extends BaseElement<DateTime> {
    async render() {
        let date = this.data!;
        this.classList.add('center')
        this.innerHTML = 
        `<h1>Week ${date.weekNumber} - ${date.year}</h1>
        <div name="items" style="display: grid; grid-gap: 10px"></div>`;

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
            })
        ]));
        
    }
}

customElements.define('weekly-planner-screen', WeeklyPlannerScreen)

export default WeeklyPlannerScreen;