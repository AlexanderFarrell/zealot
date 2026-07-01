import { BaseElementEmpty, getNavigator, getRightSidebarHost, unregisterDropZonesIn } from '@websoil/engine';
import { AttributeAPI } from '@zealot/api/src/attribute';
import { PlannerAPI } from '@zealot/api/src/planner';
import { RepeatAPI } from '@zealot/api/src/repeat';
import { icons } from '@zealot/content';
import { LoadingSpinner } from '../common/loading_spinner';
import { CommentsView } from '../views/comments_view';
import { RepeatsSidebarView } from '../views/repeats_sidebar_view';
import {
    createPlannerErrorCard,
    createPlannerHeader,
    createPlannerSection,
    currentDay,
    formatDayTitle,
    formatIsoDate,
    formatIsoWeek,
    formatMonthCode,
    formatYearCode,
    mountPlannerCardList,
    parseIsoDate,
    renderPlannerMessage,
} from './planner_shared';

const attrApi = new AttributeAPI('/api');

const plannerApi = new PlannerAPI('/api');
const repeatApi = new RepeatAPI('/api');

export class DailyPlannerScreen extends BaseElementEmpty {
    private date: string | null = null;
    private renderId = 0;
    private headerEl: HTMLElement | null = null;
    private _sidebarEl: HTMLElement | null = null;

    onActivated(): void {
        getRightSidebarHost()?.setContent(this._sidebarEl);
    }

    async render() {
        const renderId = ++this.renderId;
        const date = this.date ? parseIsoDate(this.date) : currentDay();

        this.className = 'planner-screen';
        this.innerHTML = '';
        getRightSidebarHost()?.setContent(null);

        if (!date) {
            this.appendChild(createPlannerErrorCard(
                'Invalid day. Use YYYY-MM-DD.',
                'Go to today',
                () => getNavigator().openPlanner('daily'),
            ));
            return;
        }

        if (this.headerEl) {
            unregisterDropZonesIn(this.headerEl);
            this.headerEl = null;
        }

        const header = createPlannerHeader(
            formatDayTitle(date),
            [
                {
                    iconURL: icons.back,
                    label: 'Previous Day',
                    onClick: () => getNavigator().openPlanner('daily', formatIsoDate(date.minus({ days: 1 }))),
                    onDrop: (id) => { void attrApi.set_value(id, 'Date', formatIsoDate(date.minus({ days: 1 }))); },
                },
                {
                    iconURL: icons.today,
                    label: 'Today',
                    onClick: () => getNavigator().openPlanner('daily'),
                    onDrop: (id) => { void attrApi.set_value(id, 'Date', formatIsoDate(currentDay())); },
                },
                {
                    iconURL: icons.forward,
                    label: 'Next Day',
                    onClick: () => getNavigator().openPlanner('daily', formatIsoDate(date.plus({ days: 1 }))),
                    onDrop: (id) => { void attrApi.set_value(id, 'Date', formatIsoDate(date.plus({ days: 1 }))); },
                },
            ],
            [
                {
                    iconURL: icons.week,
                    label: 'This Week',
                    onClick: () => getNavigator().openPlanner('weekly', formatIsoWeek(date)),
                },
                {
                    iconURL: icons.moon,
                    label: 'This Month',
                    onClick: () => getNavigator().openPlanner('monthly', formatMonthCode(date)),
                },
                {
                    iconURL: icons.sun,
                    label: 'This Year',
                    onClick: () => getNavigator().openPlanner('annual', formatYearCode(date)),
                },
                {
                    iconURL: icons.addNote,
                    label: 'Journal',
                    onClick: () => { getNavigator().openItem(formatIsoDate(date)); },
                },
                {
                    iconURL: icons.schedule,
                    label: 'Time Blocks',
                    onClick: () => getNavigator().openTimeBlocks('day', formatIsoDate(date)),
                },
            ],
        );

        const items = createPlannerSection('Items');
        const comments = createPlannerSection('Comments');
        items.body.appendChild(new LoadingSpinner());
        comments.body.appendChild(new CommentsView().init({
            scope: { kind: 'day', date },
        }));

        this.headerEl = header;
        this.append(header, items.section, comments.section);

        const [itemsResult, repeatsResult] = await Promise.allSettled([
            plannerApi.GetForDay(date),
            repeatApi.GetForDay(date),
        ]);

        if (renderId !== this.renderId) {
            return;
        }

        if (itemsResult.status === 'fulfilled') {
            mountPlannerCardList(items.body, {
                createRow: {
                    defaultAttributes: {
                        Date: formatIsoDate(date),
                        Priority: 3,
                        Status: 'To Do',
                    },
                    enabled: true,
                    submitLabel: 'Add Item',
                },
                emptyMessage: 'No items scheduled for this day.',
                items: itemsResult.value,
            });
        } else {
            renderPlannerMessage(items.body, this._messageForError(itemsResult.reason, 'Failed to load items.'), 'error');
        }

        if (repeatsResult.status === 'fulfilled') {
            const panel = new RepeatsSidebarView().init(repeatsResult.value, date);
            this._sidebarEl = panel;
            getRightSidebarHost()?.setContent(panel);
        } else {
            this._sidebarEl = null;
        }
    }

    disconnectedCallback(): void {
        getRightSidebarHost()?.setContent(null);
        if (this.headerEl) {
            unregisterDropZonesIn(this.headerEl);
            this.headerEl = null;
        }
    }

    init(date: string): this {
        this.date = date;
        if (this.isConnected) {
            void this.render();
        }
        return this;
    }

    private _messageForError(error: unknown, fallback: string): string {
        return error instanceof Error && error.message ? error.message : fallback;
    }
}

if (!customElements.get('daily-planner-screen')) {
    customElements.define('daily-planner-screen', DailyPlannerScreen);
}
