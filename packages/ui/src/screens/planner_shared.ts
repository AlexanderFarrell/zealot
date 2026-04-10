import { DateTime } from 'luxon';
import { icons } from '@zealot/content';
import {
    ItemTableView,
    type ItemTableColumn,
    type ItemTableCreateRowConfig,
} from '../views/item_table_view';
import type { Item } from '@zealot/domain/src/item';

export interface PlannerHeaderAction {
    iconURL: string;
    label: string;
    onClick: () => void;
}

export interface PlannerSectionElements {
    body: HTMLDivElement;
    section: HTMLElement;
}

export const plannerTableColumns: ItemTableColumn[] = [
    { kind: 'title', label: 'Title' },
    { kind: 'types', label: 'Type' },
    { attributeKey: 'Status', kind: 'attribute', label: 'Status' },
    { attributeKey: 'Priority', kind: 'attribute', label: 'Priority' },
];

export function currentDay(): DateTime {
    return DateTime.local().startOf('day');
}

export function currentWeek(): DateTime {
    const today = DateTime.local();
    return DateTime.fromObject({
        weekYear: today.weekYear,
        weekNumber: today.weekNumber,
        weekday: 1,
    }).startOf('day');
}

export function currentMonth(): DateTime {
    return DateTime.local().startOf('month');
}

export function currentYear(): DateTime {
    return DateTime.local().startOf('year');
}

export function formatIsoDate(date: DateTime): string {
    return date.toISODate() ?? date.toFormat('yyyy-MM-dd');
}

export function formatIsoWeek(date: DateTime): string {
    return date.toFormat("kkkk-'W'WW");
}

export function formatMonthCode(date: DateTime): string {
    return date.toFormat('yyyy-MM');
}

export function formatYearCode(date: DateTime): string {
    return date.toFormat('yyyy');
}

export function parseIsoDate(value: string): DateTime | null {
    const parsed = DateTime.fromFormat(value, 'yyyy-MM-dd');
    return parsed.isValid ? parsed.startOf('day') : null;
}

export function parseIsoWeek(value: string): DateTime | null {
    const match = /^(?<year>\d{4})-W(?<week>\d{2})$/.exec(value);
    if (!match?.groups) {
        return null;
    }

    const parsed = DateTime.fromObject({
        weekYear: Number(match.groups.year),
        weekNumber: Number(match.groups.week),
        weekday: 1,
    });
    return parsed.isValid ? parsed.startOf('day') : null;
}

export function parseMonthCode(value: string): DateTime | null {
    const parsed = DateTime.fromFormat(value, 'yyyy-MM');
    return parsed.isValid ? parsed.startOf('month') : null;
}

export function parseYearCode(value: string): DateTime | null {
    if (!/^\d{4}$/.test(value)) {
        return null;
    }

    const parsed = DateTime.fromObject({
        day: 1,
        month: 1,
        year: Number(value),
    });
    return parsed.isValid ? parsed.startOf('year') : null;
}

export function formatDayTitle(date: DateTime): string {
    return date.toFormat('EEEE, d LLLL yyyy');
}

export function formatWeekTitle(date: DateTime): string {
    const start = date.startOf('week');
    const end = start.plus({ days: 6 });
    return `Week ${date.weekNumber} · ${start.toFormat('d LLL')} - ${end.toFormat('d LLL yyyy')}`;
}

export function formatMonthTitle(date: DateTime): string {
    return date.toFormat('LLLL yyyy');
}

export function formatYearTitle(date: DateTime): string {
    return date.toFormat('yyyy');
}

export function createPlannerHeader(title: string, actions: PlannerHeaderAction[]): HTMLElement {
    const header = document.createElement('header');
    header.className = 'planner-header';

    const titleBlock = document.createElement('div');
    titleBlock.className = 'planner-header-title';

    const heading = document.createElement('h1');
    heading.textContent = title;
    titleBlock.appendChild(heading);

    const actionBar = document.createElement('div');
    actionBar.className = 'planner-header-actions';

    actions.forEach((action) => {
        const button = document.createElement('button');
        button.type = 'button';
        button.className = 'planner-header-button';
        button.addEventListener('click', action.onClick);

        const icon = document.createElement('img');
        icon.alt = '';
        icon.src = action.iconURL;
        button.appendChild(icon);

        const label = document.createElement('span');
        label.textContent = action.label;
        button.appendChild(label);

        actionBar.appendChild(button);
    });

    header.append(titleBlock, actionBar);
    return header;
}

export function createPlannerSection(title: string): PlannerSectionElements {
    const section = document.createElement('section');
    section.className = 'planner-section';

    const heading = document.createElement('h2');
    heading.textContent = title;
    section.appendChild(heading);

    const body = document.createElement('div');
    body.className = 'planner-section-body';
    section.appendChild(body);

    return { body, section };
}

export function createPlannerErrorCard(
    message: string,
    actionLabel: string,
    onAction: () => void,
): HTMLElement {
    const wrapper = document.createElement('section');
    wrapper.className = 'planner-error-card';

    const text = document.createElement('p');
    text.className = 'tool-error';
    text.textContent = message;
    wrapper.appendChild(text);

    const action = document.createElement('button');
    action.type = 'button';
    action.className = 'planner-header-button';
    action.addEventListener('click', onAction);

    const icon = document.createElement('img');
    icon.alt = '';
    icon.src = icons.today;
    action.appendChild(icon);

    const label = document.createElement('span');
    label.textContent = actionLabel;
    action.appendChild(label);

    wrapper.appendChild(action);
    return wrapper;
}

export function renderPlannerMessage(
    container: HTMLElement,
    message: string,
    tone: 'error' | 'muted' = 'muted',
): void {
    container.innerHTML = '';
    const paragraph = document.createElement('p');
    paragraph.className = tone === 'error' ? 'tool-error' : 'tool-muted';
    paragraph.textContent = message;
    container.appendChild(paragraph);
}

export function mountPlannerTable(
    container: HTMLElement,
    options: {
        createRow?: ItemTableCreateRowConfig;
        emptyMessage: string;
        items: Item[];
    },
): void {
    container.innerHTML = '';
    const table = new ItemTableView();
    container.appendChild(table);
    const config = {
        columns: plannerTableColumns,
        emptyMessage: options.emptyMessage,
        items: options.items,
    };

    if (options.createRow) {
        Object.assign(config, { createRow: options.createRow });
    }

    table.init(config);
}
