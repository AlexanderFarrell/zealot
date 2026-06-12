import { DateTime } from 'luxon';
import { getNavigator, registerDropZone, unregisterDropZonesIn, registerContextMenu, unregisterContextMenuIn } from '@websoil/engine';
import { AttributeAPI } from '@zealot/api/src/attribute';
import { icons } from '@zealot/content';
import {
    formatIsoDate,
    formatIsoWeek,
    formatMonthCode,
    formatMonthTitle,
    formatWeekTitle,
} from '../screens/planner_shared';

const attrApi = new AttributeAPI('/api');

const WEEKDAY_LABELS = ['S', 'M', 'T', 'W', 'T', 'F', 'S'];
const MONTH_LABELS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

export class CalendarToolView extends HTMLElement {
    private visibleMonth = DateTime.local().startOf('month');

    connectedCallback(): void {
        this.render();
    }

    disconnectedCallback(): void {
        unregisterDropZonesIn(this);
        unregisterContextMenuIn(this);
    }

    private render(): void {
        unregisterDropZonesIn(this);
        unregisterContextMenuIn(this);
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
        const panel = this.querySelector('.tool-panel');
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
            registerDropZone({
                element: button,
                onDragOver: (e) => {
                    if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
                    button.classList.add('calendar-tool-day--drop-target');
                },
                onDrop: (id) => {
                    void attrApi.set_value(id, 'Date', formatIsoDate(current));
                },
                onDragLeave: () => {
                    button.classList.remove('calendar-tool-day--drop-target');
                },
            });
            registerContextMenu(button, () => [
                { label: `Open ${formatIsoDate(current)} in Planner`, onClick: () => getNavigator().openPlanner('daily', formatIsoDate(current)) },
                { label: `Open ${formatIsoDate(current)} in Time Blocks`, onClick: () => getNavigator().openTimeBlocks('day', formatIsoDate(current)) },
            ]);
            grid.appendChild(button);
        }

        if (panel) {
            this.renderWeekButtons(panel);
            this.renderMonthButtons(panel);
        }
    }

    private renderWeekButtons(panel: Element): void {
        const firstDay = this.visibleMonth.startOf('month');
        const lastDay = this.visibleMonth.endOf('month');
        const today = DateTime.local();

        const section = document.createElement('div');
        section.className = 'calendar-tool-section';

        const label = document.createElement('div');
        label.className = 'tool-label';
        label.textContent = 'Weeks';
        section.appendChild(label);

        const weekGrid = document.createElement('div');
        weekGrid.className = 'calendar-tool-week-grid';

        let weekStart = firstDay.startOf('week');
        while (weekStart.toMillis() <= lastDay.toMillis()) {
            const w = weekStart;
            const button = document.createElement('button');
            button.type = 'button';
            button.className = 'calendar-tool-week-btn';
            button.textContent = `Wk ${w.weekNumber}`;
            button.title = formatWeekTitle(w);
            if (w.hasSame(today, 'week')) {
                button.classList.add('today');
            }
            button.addEventListener('click', () => {
                getNavigator().openPlanner('weekly', formatIsoWeek(w));
            });
            registerDropZone({
                element: button,
                onDragOver: (e) => {
                    if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
                    button.classList.add('calendar-tool-day--drop-target');
                },
                onDrop: (id) => { void attrApi.set_value(id, 'Week', formatIsoWeek(w)); },
                onDragLeave: () => {
                    button.classList.remove('calendar-tool-day--drop-target');
                },
            });
            registerContextMenu(button, () => [
                { label: `Open Week ${w.weekNumber} in Planner`, onClick: () => getNavigator().openPlanner('weekly', formatIsoWeek(w)) },
                { label: `Open Week ${w.weekNumber} in Time Blocks`, onClick: () => getNavigator().openTimeBlocks('week', formatIsoWeek(w)) },
            ]);
            weekGrid.appendChild(button);
            weekStart = weekStart.plus({ weeks: 1 });
        }

        section.appendChild(weekGrid);
        panel.appendChild(section);
    }

    private renderMonthButtons(panel: Element): void {
        const today = DateTime.local();
        const year = this.visibleMonth.year;

        const section = document.createElement('div');
        section.className = 'calendar-tool-section';

        const label = document.createElement('div');
        label.className = 'tool-label';
        label.textContent = 'Months';
        section.appendChild(label);

        const monthGrid = document.createElement('div');
        monthGrid.className = 'calendar-tool-month-grid';

        for (let m = 1; m <= 12; m += 1) {
            const monthDate = DateTime.fromObject({ year, month: m, day: 1 });
            const button = document.createElement('button');
            button.type = 'button';
            button.className = 'calendar-tool-month-btn';
            button.textContent = MONTH_LABELS[m - 1] ?? '';
            if (monthDate.hasSame(today, 'month')) {
                button.classList.add('today');
            }
            button.addEventListener('click', () => {
                getNavigator().openPlanner('monthly', formatMonthCode(monthDate));
            });
            registerDropZone({
                element: button,
                onDragOver: (e) => {
                    if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
                    button.classList.add('calendar-tool-day--drop-target');
                },
                onDrop: (id) => {
                    void Promise.all([
                        attrApi.set_value(id, 'Month', m),
                        attrApi.set_value(id, 'Year', year),
                    ]);
                },
                onDragLeave: () => {
                    button.classList.remove('calendar-tool-day--drop-target');
                },
            });
            registerContextMenu(button, () => [
                { label: `Open ${MONTH_LABELS[m - 1] ?? ''} ${year} in Planner`, onClick: () => getNavigator().openPlanner('monthly', formatMonthCode(monthDate)) },
            ]);
            monthGrid.appendChild(button);
        }

        section.appendChild(monthGrid);
        panel.appendChild(section);
    }
}

if (!customElements.get('calendar-tool-view')) {
    customElements.define('calendar-tool-view', CalendarToolView);
}
