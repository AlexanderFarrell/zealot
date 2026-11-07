import { BaseElementEmpty, MobilePaneState, Popups, getNavigator, getRightSidebarHost, unregisterDropZonesIn } from '@websoil/engine';
import { AttributeAPI } from '@zealot/api/src/attribute';
import { ItemAPI } from '@zealot/api/src/item';
import type { Item } from '@zealot/domain/src/item';
import { PlannerAPI } from '@zealot/api/src/planner';
import { RepeatAPI } from '@zealot/api/src/repeat';
import type { RepeatEntry } from '@zealot/domain/src/repeat';
import type { DateTime } from 'luxon';
import { icons } from '@zealot/content';
import { LoadingSpinner } from '../common/loading_spinner';
import { CommentsView } from '../views/comments_view';
import { RepeatsSidebarView } from '../views/repeats_sidebar_view';
import { loadAttributeKinds } from '../views/attribute_value_input';
import type { CardField } from '../views/item_card_points';
import type { ItemCardExtras } from '../views/item_card_list';
import type { AttributeKind } from '@zealot/domain/src/attribute';
import {
    type GroupBy,
    NO_VALUE_GROUP,
    OTHER_GROUP,
    carryOverWindow,
    dayScore,
    formatPoints,
    groupItems,
    isCarriedOver,
    isHeaderItem,
    isWorkItem,
} from './daily_planner_model';
import { type DailyPlannerPreferences, isGroupCollapsed, readDailyPlannerPreferences, setGroupCollapsed } from './planner_preferences';
import {
    type PlannerCardGroup,
    type PlannerCardListHandle,
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
const itemApi = new ItemAPI('/api');

const plannerApi = new PlannerAPI('/api');
const repeatApi = new RepeatAPI('/api');

export class DailyPlannerScreen extends BaseElementEmpty {
    private date: string | null = null;
    private renderId = 0;
    private headerEl: HTMLElement | null = null;
    private dayBarEl: HTMLElement | null = null;
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

        const isToday = date.hasSame(currentDay(), 'day');
        // Below the header rather than in it, so the phone header stays two lines (Z239).
        // The phone Repeats pill joins this row.
        const dayBar = document.createElement('div');
        dayBar.className = 'planner-day-bar mobile-pane-keep';
        const summary = document.createElement('div');
        summary.className = 'planner-day-summary';
        dayBar.appendChild(summary);
        this.dayBarEl = dayBar;

        // In the page only while it has cards; starts collapsed, its heading shows the count.
        const carried = createPlannerSection('Carried Over', { collapsed: true });
        carried.section.classList.add('planner-section--carried');
        const items = createPlannerSection('Items');
        const comments = createPlannerSection('Comments', { collapsed: true });
        items.body.appendChild(new LoadingSpinner());
        comments.body.appendChild(new CommentsView().init({
            scope: { kind: 'day', date },
        }));

        this.headerEl = header;
        this.append(header, dayBar, items.section, comments.section);

        const prefs = readDailyPlannerPreferences();
        const [itemsResult, repeatsResult, carriedResult, kindsResult] = await Promise.allSettled([
            plannerApi.GetForDay(date),
            repeatApi.GetForDay(date),
            isToday ? this._loadCarriedOver(date, prefs.headerTypes) : Promise.resolve([] as Item[]),
            loadAttributeKinds(),
        ]);

        if (renderId !== this.renderId) {
            return;
        }

        if (itemsResult.status === 'fulfilled') {
            const headerItems = itemsResult.value.filter((item) => isHeaderItem(item, prefs.headerTypes));
            const carriedItems = carriedResult.status === 'fulfilled' ? carriedResult.value : [];
            const kinds = kindsResult.status === 'fulfilled' ? kindsResult.value : {};
            const groupBy = resolveGroupBy(prefs.groupBy, kinds);
            const extras: ItemCardExtras = {
                pointsFrom: prefs.pointsFrom,
                fields: resolveCardFields(prefs.cardFields, kinds, prefs.pointsFrom),
                statusSelect: prefs.cardFields.includes('Status'),
                // Notes and journal entries aren't work: no points, no field chips.
                showChips: isWorkItem,
            };
            let dayList: PlannerCardListHandle | null = null;
            let carriedList: PlannerCardListHandle | null = null;
            const paintSummary = (): void => {
                this._paintSummary(summary, isToday, prefs, dayList?.items ?? [], carriedList?.items ?? [], headerItems);
            };
            const refreshBoth = (): void => {
                dayList?.refresh();
                carriedList?.refresh();
            };
            // A card dropped on a group heading takes that group's value (desktop drag only).
            const setGroupValue = (itemId: number, key: string, value: string | null): void => {
                const item = [...(dayList?.items ?? []), ...(carriedList?.items ?? [])].find((i) => i.ItemID === itemId);
                const previous: unknown = item?.Attributes[key];
                if (item) {
                    if (value === null) delete item.Attributes[key];
                    else item.Attributes[key] = value;
                    refreshBoth();
                }
                const request = value === null ? attrApi.remove(itemId, key) : attrApi.set_value(itemId, key, value);
                request.catch(() => {
                    if (item) {
                        if (previous === undefined) delete item.Attributes[key];
                        else item.Attributes[key] = previous;
                        refreshBoth();
                    }
                    Popups.add(`Could not set ${key}`);
                });
            };
            // Both lists share the collapsed groups, so a toggle in one redraws the other.
            const grouper = makeGrouper(groupBy, prefs, refreshBoth, setGroupValue);

            dayList = mountPlannerCardList(items.body, {
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
                items: itemsResult.value.filter((item) => !isHeaderItem(item, prefs.headerTypes)),
                groupBy: grouper,
                extras,
                onChange: paintSummary,
            });

            if (isToday) {
                carriedList = this._mountCarriedOver(carried, carriedItems, date, dayList, paintSummary, items.section, grouper, extras);
            }
            // The first redraws ran before the handles existed, so paint once both are set.
            paintSummary();
            if (carriedResult.status === 'rejected') {
                items.section.before(carried.section);
                renderPlannerMessage(carried.body, this._messageForError(carriedResult.reason, 'Failed to load carried-over items.'), 'error');
            }
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
     * Unfinished items that slipped from the last 14 days (Z277). The server
     * narrows by Date; the status and header-type rules run here.
     */
    private async _loadCarriedOver(date: DateTime, headerTypes: string[]): Promise<Item[]> {
        const { from, to } = carryOverWindow(date);
        const candidates = await itemApi.FilterAll([
            { key: 'Date', op: 'gte', value: from, list_mode: 'any' },
            { key: 'Date', op: 'lte', value: to, list_mode: 'any' },
        ]);
        return candidates.filter((item) => isCarriedOver(item, date, headerTypes));
    }

    /** Shows the carried-over cards; "→ Today" moves one onto the day's list. */
    private _mountCarriedOver(
        section: { body: HTMLDivElement; section: HTMLElement },
        carriedItems: Item[],
        date: DateTime,
        dayList: PlannerCardListHandle,
        paintSummary: () => void,
        itemsSection: HTMLElement,
        grouper: (items: Item[]) => PlannerCardGroup[],
        extras: ItemCardExtras,
    ): PlannerCardListHandle {
        const target = formatIsoDate(date);
        let carriedList: PlannerCardListHandle | null = null;
        const move = (item: Item, from: Item[], to: Item[]): void => {
            const index = from.indexOf(item);
            if (index >= 0) from.splice(index, 1);
            to.push(item);
            carriedList?.refresh();
            dayList.refresh();
        };
        const moveToDay = (item: Item): void => {
            if (!carriedList) return;
            const previous: unknown = item.Attributes['Date'];
            item.Attributes['Date'] = target;
            move(item, carriedList.items, dayList.items);
            attrApi.set_value(item.ItemID, 'Date', target).catch(() => {
                if (!carriedList) return;
                item.Attributes['Date'] = previous;
                move(item, dayList.items, carriedList.items);
                Popups.add(`Could not move ${item.DisplayTitle}`);
            });
        };

        carriedList = mountPlannerCardList(section.body, {
            emptyMessage: 'Nothing carried over.',
            items: carriedItems,
            groupBy: grouper,
            extras: {
                ...extras,
                action: { label: '→ Today', title: 'Move to today', onClick: moveToDay },
            },
            onChange: (list) => {
                const toggle = section.section.querySelector('.collapsible-toggle');
                if (toggle) toggle.textContent = `Carried Over (${list.length})`;
                if (list.length === 0) section.section.remove();
                else if (!section.section.isConnected) itemsSection.before(section.section);
                paintSummary();
            },
        });
        return carriedList;
    }

    /** Header line: "Today: earned / planned pts" when points are on, and a link per header item. */
    private _paintSummary(summary: HTMLElement, isToday: boolean, prefs: DailyPlannerPreferences, dayItems: Item[], carriedItems: Item[], headerItems: Item[]): void {
        summary.innerHTML = '';
        if (prefs.pointsFrom.length > 0) {
            summary.appendChild(this._buildScore(isToday, prefs.pointsFrom, dayItems, carriedItems));
        }
        for (const log of headerItems) {
            const link = document.createElement('a');
            link.className = 'planner-day-log';
            link.href = `/item/${encodeURIComponent(log.Title)}`;
            link.textContent = log.DisplayTitle;
            link.addEventListener('click', (e) => {
                e.preventDefault();
                getNavigator().openItemById(log.ItemID);
            });
            summary.appendChild(link);
        }
    }

    /** The "Today: earned / planned pts" pill. */
    private _buildScore(isToday: boolean, pointsFrom: string[], dayItems: Item[], carriedItems: Item[]): HTMLElement {
        const { earned, planned } = dayScore(dayItems, isToday ? carriedItems : [], pointsFrom);

        const score = document.createElement('span');
        score.className = 'planner-day-score';
        score.title = 'Earned: today\'s Complete items. Planned: today\'s items plus carried over.';
        const label = document.createElement('span');
        label.className = 'planner-day-score__label';
        label.textContent = isToday ? 'Today' : 'Day';
        const earnedEl = document.createElement('strong');
        earnedEl.textContent = formatPoints(earned);
        const plannedEl = document.createElement('span');
        plannedEl.textContent = ` / ${formatPoints(planned)} pts`;
        score.append(label, earnedEl, plannedEl);
        return score;
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
        if (this.dayBarEl) this.dayBarEl.appendChild(pill);
        else this.headerEl.after(pill);
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

/** The grouping attribute with its dropdown values, or null when the list is not grouped. */
function resolveGroupBy(key: string | null, kinds: Record<string, AttributeKind>): GroupBy | null {
    if (!key) return null;
    const values: unknown = kinds[key]?.Config?.values;
    return { key, values: Array.isArray(values) ? values.map(String) : [] };
}

/**
 * The "Edit on cards" fields that are dropdown or number attributes; Status is
 * the card's select instead. A points attribute with no kind is still a number.
 */
function resolveCardFields(keys: string[], kinds: Record<string, AttributeKind>, pointsFrom: string[]): CardField[] {
    const fields: CardField[] = [];
    for (const key of keys) {
        const kind = kinds[key];
        if (key === 'Status') continue;
        if (!kind) {
            if (pointsFrom.includes(key)) fields.push({ key, kind: 'number' });
            continue;
        }
        if (kind.BaseType === 'integer' || kind.BaseType === 'decimal') {
            fields.push({ key, kind: 'number' });
        } else if (kind.BaseType === 'dropdown') {
            const values: unknown = kind.Config?.values;
            fields.push({ key, kind: 'dropdown', values: Array.isArray(values) ? values.map(String) : [] });
        }
    }
    return fields;
}

/**
 * Builds the card groups for both lists (Z277). Collapse state is saved to
 * settings per grouping attribute ('' when not grouping); dropping a card on a
 * heading sets that value, and on "No <Attribute>" clears it.
 */
function makeGrouper(
    groupBy: GroupBy | null,
    prefs: DailyPlannerPreferences,
    onCollapseChange: () => void,
    setGroupValue: (itemId: number, key: string, value: string | null) => void,
): (items: Item[]) => PlannerCardGroup[] {
    const collapseKey = groupBy?.key ?? '';
    return (items) => groupItems(items, groupBy, prefs.pointsFrom).map((group) => {
        if (group.label === '') return { label: group.label, items: group.items };
        const dropTarget = groupBy && group.key !== OTHER_GROUP;
        return {
            label: group.label,
            items: group.items,
            collapsed: isGroupCollapsed(collapseKey, group.key),
            onToggle: (collapsed: boolean) => {
                setGroupCollapsed(collapseKey, group.key, collapsed);
                onCollapseChange();
            },
            ...(dropTarget ? {
                onDropItem: (itemId: number) => setGroupValue(itemId, groupBy.key, group.key === NO_VALUE_GROUP ? null : group.key),
            } : {}),
        };
    });
}

if (!customElements.get('daily-planner-screen')) {
    customElements.define('daily-planner-screen', DailyPlannerScreen);
}
