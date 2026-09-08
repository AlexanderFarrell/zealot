import { DateTime } from 'luxon';
import { getNavigator, registerDropZone, unregisterDropZone, unregisterDropZonesIn } from '@websoil/engine';
import { icons } from '@zealot/content';
import type { AttributeKind } from '@zealot/domain/src/attribute';
import type { Item } from '@zealot/domain/src/item';
import { loadAttributeKinds } from '../views/attribute_value_input';
import { buildItemCardList } from '../views/item_card_list';
import { buildAddPanel } from '../views/item_table_add_panel';
import { createItem } from '../views/item_table_save';
import type { CreateDraftState, ItemTableCreateRowConfig } from '../views/item_table_types';

export interface PlannerHeaderAction {
    iconURL: string;
    label: string;
    onClick: () => void;
    onDrop?: (draggedItemId: number) => void;
}

export interface PlannerSectionElements {
    body: HTMLDivElement;
    section: HTMLElement;
}

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

function buildActionButton(action: PlannerHeaderAction, className: string): HTMLButtonElement {
    const button = document.createElement('button');
    button.type = 'button';
    button.className = className;
    button.addEventListener('click', action.onClick);

    const icon = document.createElement('img');
    icon.alt = '';
    icon.src = action.iconURL;
    button.appendChild(icon);

    const label = document.createElement('span');
    label.textContent = action.label;
    button.appendChild(label);

    if (action.onDrop) {
        const onDrop = action.onDrop;
        registerDropZone({
            element: button,
            onDragOver: (e) => {
                if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
                button.classList.add('drop-zone--active');
            },
            onDrop: (id) => { onDrop(id); },
            onDragLeave: () => {
                button.classList.remove('drop-zone--active');
            },
        });
    }

    return button;
}

export function createPlannerHeader(
    title: string,
    actions: PlannerHeaderAction[],
    crossLinks?: PlannerHeaderAction[],
): HTMLElement {
    const header = document.createElement('header');
    header.className = 'planner-header';

    const heading = document.createElement('h1');
    heading.className = 'planner-header-title';
    heading.textContent = title;

    const actionBar = document.createElement('div');
    actionBar.className = 'planner-header-actions';
    actions.forEach((action) => {
        actionBar.appendChild(buildActionButton(action, 'planner-header-button'));
    });

    if (crossLinks && crossLinks.length > 0) {
        const sep = document.createElement('span');
        sep.className = 'planner-header-sep';
        sep.setAttribute('aria-hidden', 'true');
        actionBar.appendChild(sep);

        crossLinks.forEach((action) => {
            actionBar.appendChild(buildActionButton(action, 'planner-header-link'));
        });
    }

    header.append(heading, actionBar);
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

export function mountPlannerCardList(
    container: HTMLElement,
    options: {
        createRow?: Pick<ItemTableCreateRowConfig, 'defaultAttributes' | 'submitLabel' | 'enabled'>;
        emptyMessage: string;
        items: Item[];
    },
): void {
    container.innerHTML = '';

    const items = [...options.items];
    const { emptyMessage } = options;

    const cardListDiv = document.createElement('div');
    container.appendChild(cardListDiv);

    const refreshCardList = (): void => {
        unregisterDropZonesIn(cardListDiv);
        cardListDiv.innerHTML = '';
        cardListDiv.appendChild(buildItemCardList(items, emptyMessage, { grouped: true, showParent: true, onDrop: refreshCardList }));
    };

    if (!options.createRow) {
        refreshCardList();
        return;
    }

    const createRowConfig = options.createRow;
    const defaultAttributes = { ...createRowConfig.defaultAttributes };

    const newDraft = (): CreateDraftState => ({
        attributes: { ...defaultAttributes },
        title: '',
        types: [],
    });

    let draft = newDraft();
    let attributeKinds: Record<string, AttributeKind> = {};
    let panelEl: HTMLElement | null = null;

    const rebuildPanel = (startOpen: boolean): void => {
        const fullConfig: ItemTableCreateRowConfig = { ...createRowConfig, enabled: true };
        const nextPanel = buildAddPanel(
            fullConfig,
            draft,
            attributeKinds,
            startOpen,
            onSubmit,
            onReset,
        );
        if (panelEl) {
            panelEl.replaceWith(nextPanel);
        } else {
            container.insertBefore(nextPanel, cardListDiv);
        }
        panelEl = nextPanel;
    };

    const onSubmit = (): void => {
        const title = draft.title.trim();
        if (!title) return;
        void createItem({
            attributes: draft.attributes,
            title,
            types: draft.types,
        }).then((item) => {
            items.push(item);
            refreshCardList();
            draft = newDraft();
            rebuildPanel(true);
        });
    };

    const onReset = (): void => {
        draft = newDraft();
    };

    refreshCardList();
    rebuildPanel(false);

    void loadAttributeKinds().then((kinds) => {
        attributeKinds = kinds;
        const isOpen = panelEl !== null && !panelEl.classList.contains('item-table-add-panel--collapsed');
        rebuildPanel(isOpen);
    });
}

export function journalTitleForWeek(date: DateTime): string {
    return `Week ${date.weekNumber} ${date.weekYear}`;
}
