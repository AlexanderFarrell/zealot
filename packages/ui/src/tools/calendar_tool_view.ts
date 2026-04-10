import { DateTime } from 'luxon';
import { getNavigator } from '@websoil/engine';
import { icons } from '@zealot/content';

const WEEKDAY_LABELS = ['S', 'M', 'T', 'W', 'T', 'F', 'S'];

function formatIsoDate(date: DateTime): string {
    return date.toISODate() ?? date.toFormat('yyyy-MM-dd');
}

function formatMonthTitle(date: DateTime): string {
    return date.toFormat('LLLL yyyy');
}

export class CalendarToolView extends HTMLElement {
    private visibleMonth = DateTime.local().startOf('month');

    connectedCallback(): void {
        this.render();
    }

    private render(): void {
        this.innerHTML = `
        <div class="tool-panel">
            <div class="tool-panel-header tool-panel-header-spread">
                <button type="button" class="calendar-tool-nav" aria-label="Previous month">
                    <img src="${icons.back}" alt="">
                </button>
                <h2>${formatMonthTitle(this.visibleMonth)}</h2>
                <button type="button" class="calendar-tool-nav" aria-label="Next month">
                    <img src="${icons.forward}" alt="">
                </button>
            </div>
            <button type="button" class="calendar-tool-today">Today</button>
            <div class="calendar-tool-grid">
                ${WEEKDAY_LABELS.map((label) => `<div class="calendar-tool-weekday">${label}</div>`).join('')}
            </div>
        </div>
        `;

        const previousButton = this.querySelector('.calendar-tool-nav[aria-label="Previous month"]');
        const nextButton = this.querySelector('.calendar-tool-nav[aria-label="Next month"]');
        const todayButton = this.querySelector('.calendar-tool-today');
        const grid = this.querySelector('.calendar-tool-grid');

        previousButton?.addEventListener('click', () => {
            this.visibleMonth = this.visibleMonth.minus({ months: 1 }).startOf('month');
            this.render();
        });

        nextButton?.addEventListener('click', () => {
            this.visibleMonth = this.visibleMonth.plus({ months: 1 }).startOf('month');
            this.render();
        });

        todayButton?.addEventListener('click', () => {
            this.visibleMonth = DateTime.local().startOf('month');
            this.render();
        });

        if (!grid) {
            return;
        }

        const firstDay = this.visibleMonth.startOf('month');
        const leadingDays = firstDay.weekday % 7;
        const daysInMonth = this.visibleMonth.daysInMonth;
        const today = DateTime.local();

        for (let index = 0; index < leadingDays; index += 1) {
            const spacer = document.createElement('div');
            spacer.className = 'calendar-tool-day is-empty';
            grid.appendChild(spacer);
        }

        for (let day = 1; day <= daysInMonth; day += 1) {
            const current = this.visibleMonth.set({ day });
            const button = document.createElement('button');
            button.type = 'button';
            button.className = 'calendar-tool-day';
            button.textContent = String(day);
            if (current.hasSame(today, 'day')) {
                button.classList.add('today');
            }
            button.addEventListener('click', () => {
                getNavigator().openPlanner('daily', formatIsoDate(current));
            });
            grid.appendChild(button);
        }
    }
}

if (!customElements.get('calendar-tool-view')) {
    customElements.define('calendar-tool-view', CalendarToolView);
}
