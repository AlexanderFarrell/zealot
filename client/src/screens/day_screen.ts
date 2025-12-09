import { DateTime } from "luxon";
import API from "../api/api";
import PlanView from "../components/plan_view";
import { router } from "../core/router";
import HomeIcon from "../assets/icon/home.svg";
import PreviousIcon from "../assets/icon/back.svg";
import NextIcon from "../assets/icon/forward.svg";
import WeekIcon from "../assets/icon/week.svg";
import MonthIcon from "../assets/icon/moon.svg";
import YearIcon from "../assets/icon/sun.svg";
import DocIcon from "../assets/icon/doc.svg";
import BaseElement from "../components/common/base_element";
import ButtonGroup, { ButtonDef } from "../components/common/button_group";


class DailyPlannerScreen extends BaseElement<DateTime> {
    async render() {
        let date = this.data!;
        this.classList.add('center')
        this.innerHTML = `
        <h1>${date.toFormat(`EEEE - d MMMM yyyy`)}</h1>
        <div name="items" style="display: grid; grid-gap: 10px"></div>`
        this.prepend(new ButtonGroup().init([
            new ButtonDef(HomeIcon, "Today", () => {
                let today = DateTime.now()
                router.navigate(`/planner/daily/${today.toISODate()}`);
            }),
            new ButtonDef(PreviousIcon, "Previous Day", () => {
                let previous = date.minus({days: 1})
                router.navigate(`/planner/daily/${previous.toISODate()}`)
            }),
            new ButtonDef(NextIcon, "Next Day", () => {
                let next = date.plus({days: 1});
                router.navigate(`/planner/daily/${next.toISODate()}`)
            }),
            new ButtonDef(WeekIcon, `Week ${date.weekNumber} - ${date.year}`, () => {
                let week = date.toISOWeekDate()?.substring(0, 8);
                router.navigate(`/planner/weekly/${week}`)
            }),
            new ButtonDef(MonthIcon, `${date.monthLong} ${date.year}`, () => {
                let month_str = date.toFormat(`yyyy-MM`)
                router.navigate(`/planner/monthly/${month_str}`);
            }),
            new ButtonDef(YearIcon, `${date.year}`, () => {
                router.navigate(`/planner/annual/${date.year}`)
            }),
            new ButtonDef(DocIcon, 'Create Note', () => {
                router.navigate(`/item/${date!.toISODate()}`)
            })
        ]))
        let items_container = this.querySelector('[name="items"]')! as HTMLElement;

        try {
            let items = await API.planner.get_items_on_day(date!);
            items.forEach(item => {
                let view = new PlanView();
                view.item = item;
                items_container.appendChild(view)
            })
            if (items.length == 0) {
                items_container.innerHTML = "No items scheduled for this day."
            }
        } catch (e) {
            console.error(e)
            items_container.innerHTML = "<div class='error'>Error getting items</div>"
        }
    }
}

customElements.define('daily-planner-screen', DailyPlannerScreen)

export default DailyPlannerScreen;