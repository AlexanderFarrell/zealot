import { BaseElementEmpty, MobilePaneState, getNavigator, getRightSidebarHost, unregisterDropZonesIn } from '@websoil/engine';
import { AttributeAPI } from '@zealot/api/src/attribute';
import { PlannerAPI } from '@zealot/api/src/planner';
import { RepeatAPI } from '@zealot/api/src/repeat';
import type { RepeatEntry } from '@zealot/domain/src/repeat';
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
    private _repeatCount: number | null = null;
    private _unsubscribePane: (() => void) | null = null;

    onActivated(): void {
        getRightSidebarHost()?.setContent(this._sidebarEl, { label: 'Repeats', count: this._repeatCount });
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
        const comments = createPlannerSection('Comments', { collapsedOnMobile: true });
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
            this._repeatCount = repeatsResult.value.length;
            this._mountRepeatPill(repeatsResult.value, panel);
            getRightSidebarHost()?.setContent(panel, { label: 'Repeats', count: repeatsResult.value.length });
        } else {
            this._sidebarEl = null;
        }
    }

    /**
     * Phone-only pill under the header (Z238). It reads `Repeats 3/27` on the
     * day's items and `← Items` in the Repeats view, so both pane switches use
     * the same spot.
     */
    private _mountRepeatPill(entries: RepeatEntry[], panel: HTMLElement): void {
        if (entries.length === 0 || !this.headerEl) return;

        const pill = document.createElement('button');
        pill.type = 'button';
        pill.className = 'planner-repeat-progress mobile-pane-keep';
        const paint = (): void => {
            if (MobilePaneState.getPane() === 'side') {
                pill.textContent = '← Items';
                pill.classList.remove('is-done');
                return;
            }
            const done = entries.filter((entry) => entry.Status === 'Complete').length;
            pill.textContent = `Repeats ${done}/${entries.length}`;
            pill.classList.toggle('is-done', done === entries.length);
        };
        paint();
        pill.addEventListener('click', () => {
            MobilePaneState.setPane(MobilePaneState.getPane() === 'side' ? 'content' : 'side');
        });
        panel.addEventListener('repeat-status-change', paint);
        this._unsubscribePane?.();
        this._unsubscribePane = MobilePaneState.subscribe(paint);
        this.headerEl.after(pill);
    }

    disconnectedCallback(): void {
        this._unsubscribePane?.();
        this._unsubscribePane = null;
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
