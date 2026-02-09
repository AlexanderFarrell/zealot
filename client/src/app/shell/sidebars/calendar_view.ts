import { DateTime } from "luxon";
import { router } from "../../../features/router/router";
import DragUtil from "../../../features/item/drag_helper";
import { HomeIcon, BackIcon, ForwardIcon } from "../../../assets/asset_map";

class CalendarView extends HTMLElement {
    private start_date: DateTime = DateTime.fromObject({
        year: DateTime.now().year,
        month: DateTime.now().month,
        day: 1
    });

    connectedCallback() {
        this.render();
    }

    render() {
        
        let current_date = this.start_date;
        this.innerHTML = `
        <div class="row gap">
            <button name="previous">
                <img style="width: 1em" src="${BackIcon}">
            </button>
            <button name="current">
                <img style="width: 1em" src="${HomeIcon}"
            </button>
            <button name="next">
                <img style="width: 1em" src="${ForwardIcon}">
            </button>
        </div>
        <div name='title'>
        <h1>
            <span name='month_display'>${this.start_date.monthLong}</span>
            <span name='year_display'>${this.start_date.year}</span>
        </h1>
        </div>
        <div name='calendar_items'>
            <div>&nbsp;</div>
            <div>S</div>
            <div>M</div>
            <div>T</div>
            <div>W</div>
            <div>T</div>
            <div>F</div>
            <div>S</div>
        </div>`;

        let prev_button = document.querySelector('[name="previous"]')! as HTMLElement;
        let current_button = document.querySelector('[name="current"]')! as HTMLElement;
        let next_button = document.querySelector('[name="next"]')! as HTMLElement;
        let month_display = document.querySelector('[name="month_display"]')! as HTMLElement;
        let year_display = document.querySelector('[name="year_display"]')! as HTMLElement;

        prev_button.addEventListener('click', () => {
            this.start_date = this.start_date.minus({month: 1});
            this.render();
        })
        current_button.addEventListener('click', () => {
           this.start_date = DateTime.fromObject({
                year: DateTime.now().year,
                month: DateTime.now().month,
                day: 1
            }) 
            this.render()
        })

        next_button.addEventListener('click', () => {
            this.start_date = this.start_date.plus({month: 1});
            this.render();
        })

        month_display.addEventListener('click', () => {
            router.navigate(`/planner/monthly/${this.start_date.toFormat('yyyy-MM')}`)
        })

        year_display.addEventListener('click', () => {
            router.navigate(`/planner/annual/${this.start_date.year}`)
        })

        let calendar_items = document.querySelector('[name="calendar_items"]')!
        let days_before_month = this.start_date.weekday % 7;
        let days_in_month = this.start_date.daysInMonth!;
        let current_space = 0;

        let add_week_button = (day: DateTime = current_date) => {
            let str = day.toISOWeekDate()!.substring(0, 8);
            let week_view = document.createElement('button');
            week_view.innerText = day.weekNumber.toString();
            week_view.addEventListener('click', () => {
                router.navigate(`/planner/weekly/${str}`)
            })
            DragUtil.setup_drop(week_view, {
                Week: str
            })
            calendar_items.appendChild(week_view);
        }

        add_week_button(current_date.minus({day: days_before_month}));

        // Add empty spaces
        for (let i = 0; i < days_before_month; i++) {
            let empty_view = document.createElement('div');
            empty_view.innerHTML = "&nbsp;";
            calendar_items.appendChild(empty_view);
            current_space++;
        }

        let today = DateTime.fromObject({
            year: DateTime.now().year,
            month: DateTime.now().month,
            day: DateTime.now().day,
        })
        // Add day links
        for (let i = 0; i < days_in_month; i++) {
            let view = document.createElement('button');
            view.innerText = current_date.day.toString();
            let str = current_date.toISODate();
            view.addEventListener('click', () => {
                router.navigate(`/planner/daily/${str}`)
            })
            DragUtil.setup_drop(view, {
                Date: str
            })

            // if today, then emphasize
            if (current_date.day == today.day &&
                current_date.month == today.month &&
                current_date.year == today.year
            ) {
                view.classList.add('today');
            }
            calendar_items.appendChild(view);
            current_space++;
            current_date = current_date.plus({day: 1}); 
            if (current_space % 7 == 0) {
                add_week_button()
            }
        }
    }
}

customElements.define('calendar-view', CalendarView)

export default CalendarView;