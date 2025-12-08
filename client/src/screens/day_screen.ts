import type { DateTime } from "luxon";
import API from "../api/api";
import PlanView from "../components/plan_view";
import { router } from "../core/router";
import HomeIcon from "../assets/icon/home.svg";
import PreviousIcon from "../assets/icon/back.svg";
import NextIcon from "../assets/icon/forward.svg";
import DocIcon from "../assets/icon/doc.svg";

class DailyPlannerScreen extends HTMLElement {
    private _date: DateTime | null = null;

    public get date(): DateTime | null {
        return this._date;
    }

    public set date(value: DateTime) {
        this._date = value;
        this.render();
    }

    async render() {
        this.innerHTML = `
        <h1>${this.date!.toFormat(`EEEE - d MMMM yyyy`)}</h1>
        <div class="button_row" style="margin-bottom: 1em">
            <button name="today"><img style="width: 2em" src="${HomeIcon}"></button>
            <button name="prev"><img style="width: 2em" src="${PreviousIcon}"></button>
            <button name="next"><img style="width: 2em" src="${NextIcon}"></button>
            <button name="note"><img style="width: 2em" src="${DocIcon}"></button>
        </div>

        <div name="items" style="display: grid; grid-gap: 10px"></div>
        `
        let items_container = this.querySelector('[name="items"]')! as HTMLElement;

        let prev_button = this.querySelector('[name="prev"]')! as HTMLButtonElement;
        let next_button = this.querySelector('[name="next"]')! as HTMLButtonElement;
        let note_button = this.querySelector('[name="note"]')! as HTMLButtonElement;

        prev_button.addEventListener('click', () => {
            let yesterday = this.date!.minus({days: 1});
            router.navigate(`/planner/daily/${yesterday.toISODate()}`)
        })
        next_button.addEventListener('click', () => {
            let tomorrow = this.date!.plus({days: 1});
            router.navigate(`/planner/daily/${tomorrow.toISODate()}`)
        })
        note_button.addEventListener('click', () => {
            router.navigate(`/item/${this.date!.toISODate()!}`)
        })

        this.classList.add('center')
        try {
            let items = await API.planner.get_items_on_day(this.date!);
            items.forEach(item => {
                let view = new PlanView();
                view.item = item;
                items_container.appendChild(view)
            })
        } catch (e) {
            items_container.innerHTML = "<div class='error'>Error getting items</div>"
        }
    }
}

customElements.define('daily-planner-screen', DailyPlannerScreen)

export default DailyPlannerScreen;