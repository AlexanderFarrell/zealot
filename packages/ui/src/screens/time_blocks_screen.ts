import { DateTime } from 'luxon';
import { BaseElementEmpty, getNavigator } from '@websoil/engine';
import { TimeBlockAPI } from '@zealot/api/src/time_block';
import { icons } from '@zealot/content';
import type { Item } from '@zealot/domain/src/item';
import type { TimeBlock } from '@zealot/domain/src/time_block';
import type { UpdateTimeBlockDto } from '@zealot/domain/src/time_block';
import { ItemSearchInline } from '../views/item_search_inline';
import {
    createPlannerHeader,
    currentDay,
    formatIsoDate,
    formatIsoWeek,
    parseIsoDate,
    parseIsoWeek,
} from './planner_shared';

export type TimeBlockView = 'day' | '3day' | 'week' | 'item';

const api = new TimeBlockAPI('/api');

const GRID_START_HOUR = 6;
const GRID_END_HOUR = 23;
const HOUR_HEIGHT_PX = 60;
const MIN_BLOCK_MINS = 15;
const DRAG_THRESHOLD_PX = 5;

function minsToY(mins: number): number {
    return ((mins - GRID_START_HOUR * 60) / 60) * HOUR_HEIGHT_PX;
}

function yToMins(y: number): number {
    return Math.round(((y / HOUR_HEIGHT_PX) * 60 + GRID_START_HOUR * 60) / MIN_BLOCK_MINS) * MIN_BLOCK_MINS;
}

function clampMins(m: number): number {
    return Math.max(GRID_START_HOUR * 60, Math.min(GRID_END_HOUR * 60 - MIN_BLOCK_MINS, m));
}

function minsToLabel(mins: number): string {
    const h = Math.floor(mins / 60).toString().padStart(2, '0');
    const m = (mins % 60).toString().padStart(2, '0');
    return `${h}:${m}`;
}

export class TimeBlocksScreen extends BaseElementEmpty {
    private view: TimeBlockView = 'day';
    private date: string | null = null;
    private itemId: number | null = null;
    private renderId = 0;

    init(view: TimeBlockView, date: string): this {
        this.view = view;
        this.date = date;
        if (this.isConnected) void this.render();
        return this;
    }

    initForItem(itemId: number): this {
        this.view = 'item';
        this.itemId = itemId;
        if (this.isConnected) void this.render();
        return this;
    }

    connectedCallback() {
        super.connectedCallback?.();
        void this.render();
    }

    async render() {
        const renderId = ++this.renderId;
        this.className = 'time-blocks-screen';
        this.innerHTML = '';

        if (this.view === 'item' && this.itemId != null) {
            await this.renderItemView(renderId, this.itemId);
        } else {
            await this.renderGridView(renderId);
        }
    }

    private async renderGridView(renderId: number) {
        const view = this.view as 'day' | '3day' | 'week';
        const anchor = this.date
            ? (view === 'week' ? parseIsoWeek(this.date) : parseIsoDate(this.date))
            : currentDay();
        if (!anchor) {
            this.appendChild(this.errorCard('Invalid date.', 'Go to today', () =>
                getNavigator().openTimeBlocks('day'),
            ));
            return;
        }

        const { days, prevDate, nextDate, title } = this.resolveRange(view, anchor);
        const start = days[0]!;
        const end = days[days.length - 1]!;

        const header = createPlannerHeader(
            title,
            [
                {
                    iconURL: icons.back,
                    label: 'Previous',
                    onClick: () => getNavigator().openTimeBlocks(view, prevDate),
                },
                {
                    iconURL: icons.today,
                    label: 'Today',
                    onClick: () => getNavigator().openTimeBlocks(view),
                },
                {
                    iconURL: icons.forward,
                    label: 'Next',
                    onClick: () => getNavigator().openTimeBlocks(view, nextDate),
                },
            ],
            [
                {
                    iconURL: icons.schedule,
                    label: 'Day',
                    onClick: () => getNavigator().openTimeBlocks('day', formatIsoDate(anchor)),
                },
                {
                    iconURL: icons.schedule,
                    label: '3 Days',
                    onClick: () => getNavigator().openTimeBlocks('3day', formatIsoDate(anchor)),
                },
                {
                    iconURL: icons.schedule,
                    label: 'Week',
                    onClick: () => getNavigator().openTimeBlocks('week', formatIsoWeek(anchor)),
                },
            ],
        );

        // Mark active view button
        const crossLinkBtns = header.querySelectorAll<HTMLButtonElement>('.planner-header-link');
        const viewLabels: Record<string, string> = { day: 'Day', '3day': '3 Days', week: 'Week' };
        crossLinkBtns.forEach(btn => {
            if (btn.querySelector('span')?.textContent === viewLabels[view]) {
                btn.classList.add('planner-header-link--active');
            }
        });

        this.appendChild(header);

        const loading = document.createElement('p');
        loading.className = 'planner-message';
        loading.textContent = 'Loading…';
        this.appendChild(loading);

        let blocks: TimeBlock[];
        try {
            blocks = await api.GetForRange(start, end);
        } catch {
            if (renderId !== this.renderId) return;
            loading.remove();
            this.appendChild(this.errorCard('Failed to load time blocks.', 'Retry', () => void this.render()));
            return;
        }

        if (renderId !== this.renderId) return;
        loading.remove();
        const grid = this.buildGrid(days, blocks);
        this.appendChild(grid);
    }

    private async renderItemView(renderId: number, itemId: number) {
        const header = createPlannerHeader('Time Blocks for Item', []);
        this.appendChild(header);

        const loading = document.createElement('p');
        loading.className = 'planner-message';
        loading.textContent = 'Loading…';
        this.appendChild(loading);

        let blocks: TimeBlock[];
        try {
            blocks = await api.GetForItem(itemId);
        } catch {
            if (renderId !== this.renderId) return;
            loading.remove();
            this.appendChild(this.errorCard('Failed to load time blocks.', 'Retry', () => void this.render()));
            return;
        }

        if (renderId !== this.renderId) return;
        loading.remove();

        if (blocks.length === 0) {
            const msg = document.createElement('p');
            msg.className = 'planner-message';
            msg.textContent = 'No time blocks for this item.';
            this.appendChild(msg);
            return;
        }

        const grouped = new Map<string, TimeBlock[]>();
        for (const b of blocks) {
            const key = b.Date.toISODate() ?? '';
            if (!grouped.has(key)) grouped.set(key, []);
            grouped.get(key)!.push(b);
        }

        const list = document.createElement('div');
        list.className = 'time-blocks-item-list';
        for (const [dateKey, dayBlocks] of [...grouped.entries()].sort()) {
            const section = document.createElement('section');
            const heading = document.createElement('h2');
            heading.textContent = DateTime.fromISO(dateKey).toFormat('EEEE, d LLLL yyyy');
            section.appendChild(heading);
            for (const block of dayBlocks) {
                section.appendChild(this.buildBlockCard(block, () => void this.render()));
            }
            list.appendChild(section);
        }
        this.appendChild(list);
    }

    private resolveRange(view: 'day' | '3day' | 'week', anchor: DateTime): {
        days: DateTime[];
        prevDate: string;
        nextDate: string;
        title: string;
    } {
        if (view === 'day') {
            return {
                days: [anchor],
                prevDate: formatIsoDate(anchor.minus({ days: 1 })),
                nextDate: formatIsoDate(anchor.plus({ days: 1 })),
                title: anchor.toFormat('EEEE, d LLLL yyyy'),
            };
        }
        if (view === '3day') {
            return {
                days: [anchor.minus({ days: 1 }), anchor, anchor.plus({ days: 1 })],
                prevDate: formatIsoDate(anchor.minus({ days: 3 })),
                nextDate: formatIsoDate(anchor.plus({ days: 3 })),
                title: `${anchor.minus({ days: 1 }).toFormat('d LLL')} – ${anchor.plus({ days: 1 }).toFormat('d LLL yyyy')}`,
            };
        }
        const weekStart = anchor.startOf('week');
        const days: DateTime[] = [];
        for (let i = 0; i < 7; i++) days.push(weekStart.plus({ days: i }));
        return {
            days,
            prevDate: formatIsoWeek(anchor.minus({ weeks: 1 })),
            nextDate: formatIsoWeek(anchor.plus({ weeks: 1 })),
            title: `Week ${anchor.weekNumber} · ${weekStart.toFormat('d LLL')} – ${weekStart.plus({ days: 6 }).toFormat('d LLL yyyy')}`,
        };
    }

    private buildGrid(days: DateTime[], initialBlocks: TimeBlock[]): HTMLElement {
        const totalHours = GRID_END_HOUR - GRID_START_HOUR;
        const blocksByDate = new Map<string, TimeBlock[]>();
        for (const b of initialBlocks) {
            const key = b.Date.toISODate() ?? '';
            if (!blocksByDate.has(key)) blocksByDate.set(key, []);
            blocksByDate.get(key)!.push(b);
        }

        const wrapper = document.createElement('div');
        wrapper.className = 'time-blocks-grid-wrapper';

        const grid = document.createElement('div');
        grid.className = 'time-blocks-grid';
        grid.style.cssText = `grid-template-columns: 3rem repeat(${days.length}, 1fr);`;

        // Corner cell
        const corner = document.createElement('div');
        corner.className = 'time-blocks-grid-corner';
        grid.appendChild(corner);

        // Day header cells
        for (const day of days) {
            const cell = document.createElement('div');
            cell.className = 'time-blocks-day-header';
            const lbl = document.createElement('span');
            lbl.textContent = day.toFormat('EEE d');
            if (day.hasSame(DateTime.local(), 'day')) lbl.classList.add('is-today');
            cell.appendChild(lbl);
            grid.appendChild(cell);
        }

        // Time label column
        const timeCol = document.createElement('div');
        timeCol.className = 'time-blocks-time-col';
        timeCol.style.height = `${totalHours * HOUR_HEIGHT_PX}px`;
        for (let h = GRID_START_HOUR; h < GRID_END_HOUR; h++) {
            const lbl = document.createElement('div');
            lbl.className = 'time-blocks-hour-label';
            lbl.style.top = `${(h - GRID_START_HOUR) * HOUR_HEIGHT_PX}px`;
            lbl.textContent = `${h.toString().padStart(2, '0')}:00`;
            timeCol.appendChild(lbl);
        }
        grid.appendChild(timeCol);

        // Build refresh map so cross-column moves can trigger the target column's refresh
        const refreshColMap = new Map<string, () => void>();

        // Day columns
        for (const day of days) {
            const col = this.buildDayColumn(day, totalHours, blocksByDate, refreshColMap);
            grid.appendChild(col);
        }

        wrapper.appendChild(grid);
        return wrapper;
    }

    private buildDayColumn(
        day: DateTime,
        totalHours: number,
        blocksByDate: Map<string, TimeBlock[]>,
        refreshColMap: Map<string, () => void>,
    ): HTMLElement {
        const totalPx = totalHours * HOUR_HEIGHT_PX;
        const col = document.createElement('div');
        col.className = 'time-blocks-day-col';
        col.style.height = `${totalPx}px`;

        const dayKey = day.toISODate() ?? '';
        col.dataset.date = dayKey;

        // Hour separator lines
        for (let h = 0; h < totalHours; h++) {
            const sep = document.createElement('div');
            sep.className = 'time-blocks-hour-sep';
            sep.style.top = `${h * HOUR_HEIGHT_PX}px`;
            col.appendChild(sep);
        }

        const refreshCol = () => {
            col.querySelectorAll('.time-block-card').forEach(el => el.remove());
            const current = blocksByDate.get(dayKey) ?? [];
            for (const block of current) {
                col.appendChild(this.positionedBlockCard(block, col, blocksByDate, dayKey, refreshCol, refreshColMap));
            }
        };

        // Register this column's refresh so other columns can call it for cross-day moves
        refreshColMap.set(dayKey, refreshCol);

        const dayBlocks = blocksByDate.get(dayKey) ?? [];
        for (const block of dayBlocks) {
            col.appendChild(this.positionedBlockCard(block, col, blocksByDate, dayKey, refreshCol, refreshColMap));
        }

        // Drag-to-create: track drag distance to avoid opening modal on plain click
        let dragState: {
            startY: number;
            startClientY: number;
            ghost: HTMLElement;
            dragged: boolean;
        } | null = null;

        const onMouseMove = (e: MouseEvent) => {
            if (!dragState) return;
            if (Math.abs(e.clientY - dragState.startClientY) > DRAG_THRESHOLD_PX) {
                dragState.dragged = true;
            }
            const rect = col.getBoundingClientRect();
            const y = Math.max(0, Math.min(e.clientY - rect.top, rect.height));
            const top = Math.min(dragState.startY, y);
            const height = Math.abs(y - dragState.startY);
            dragState.ghost.style.top = `${top}px`;
            dragState.ghost.style.height = `${Math.max(4, height)}px`;
        };

        const onMouseUp = (e: MouseEvent) => {
            document.removeEventListener('mousemove', onMouseMove);
            document.removeEventListener('mouseup', onMouseUp);
            if (!dragState) return;
            const { dragged, ghost } = dragState;
            ghost.remove();

            if (!dragged) {
                dragState = null;
                return;
            }

            const rect = col.getBoundingClientRect();
            const endY = Math.max(0, Math.min(e.clientY - rect.top, rect.height));
            const rawStart = yToMins(Math.min(dragState.startY, endY));
            const rawEnd = yToMins(Math.max(dragState.startY, endY));
            const startMins = clampMins(rawStart);
            const endMins = clampMins(Math.max(rawEnd, startMins + MIN_BLOCK_MINS));
            dragState = null;

            this.openCreateModal(day, startMins, endMins, blocksByDate, dayKey, refreshCol);
        };

        col.addEventListener('mousedown', (e) => {
            if ((e.target as HTMLElement).closest('.time-block-card')) return;
            e.preventDefault();
            const rect = col.getBoundingClientRect();
            const startY = Math.max(0, e.clientY - rect.top);

            const ghost = document.createElement('div');
            ghost.className = 'time-block-ghost';
            ghost.style.top = `${startY}px`;
            ghost.style.height = '4px';
            col.appendChild(ghost);

            dragState = { startY, startClientY: e.clientY, ghost, dragged: false };
            document.addEventListener('mousemove', onMouseMove);
            document.addEventListener('mouseup', onMouseUp);
        });

        return col;
    }

    private positionedBlockCard(
        block: TimeBlock,
        col: HTMLElement,
        blocksByDate: Map<string, TimeBlock[]>,
        dayKey: string,
        refreshCol: () => void,
        refreshColMap: Map<string, () => void>,
    ): HTMLElement {
        const topPx = minsToY(Math.max(block.StartMin, GRID_START_HOUR * 60));
        const heightPx = Math.max(18, minsToY(block.EndMin) - minsToY(block.StartMin));

        const card = this.buildBlockCard(block, async () => {
            await api.Delete(block.BlockId);
            const arr = blocksByDate.get(dayKey) ?? [];
            blocksByDate.set(dayKey, arr.filter(b => b.BlockId !== block.BlockId));
            refreshCol();
        });
        card.style.cssText += `position:absolute;top:${topPx}px;left:2px;right:2px;height:${heightPx}px;overflow:hidden;`;

        // Top resize handle
        const topHandle = document.createElement('div');
        topHandle.className = 'time-block-resize-handle time-block-resize-handle--top';
        card.appendChild(topHandle);

        // Bottom resize handle
        const bottomHandle = document.createElement('div');
        bottomHandle.className = 'time-block-resize-handle time-block-resize-handle--bottom';
        card.appendChild(bottomHandle);

        // Move handle (middle area)
        const moveHandle = document.createElement('div');
        moveHandle.className = 'time-block-move-handle';
        card.appendChild(moveHandle);

        const makeResizeDrag = (edge: 'top' | 'bottom') => (e: MouseEvent) => {
            e.preventDefault();
            e.stopPropagation();
            const colRect = col.getBoundingClientRect();
            let currentStart = block.StartMin;
            let currentEnd = block.EndMin;

            const onMove = (ev: MouseEvent) => {
                const y = Math.max(0, ev.clientY - colRect.top);
                const mins = clampMins(yToMins(y));
                if (edge === 'top') {
                    currentStart = Math.min(mins, currentEnd - MIN_BLOCK_MINS);
                } else {
                    currentEnd = Math.max(mins, currentStart + MIN_BLOCK_MINS);
                }
                const newTop = minsToY(currentStart);
                const newH = Math.max(18, minsToY(currentEnd) - minsToY(currentStart));
                card.style.top = `${newTop}px`;
                card.style.height = `${newH}px`;
                const timeEl = card.querySelector<HTMLSpanElement>('.time-block-time');
                if (timeEl) timeEl.textContent = `${minsToLabel(currentStart)}–${minsToLabel(currentEnd)} `;
            };

            const onUp = async () => {
                document.removeEventListener('mousemove', onMove);
                document.removeEventListener('mouseup', onUp);
                try {
                    await api.Update({ block_id: block.BlockId, start_min: currentStart, end_min: currentEnd });
                    block.StartMin = currentStart;
                    block.EndMin = currentEnd;
                } catch {
                    refreshCol();
                }
            };

            document.addEventListener('mousemove', onMove);
            document.addEventListener('mouseup', onUp);
        };

        topHandle.addEventListener('mousedown', makeResizeDrag('top'));
        bottomHandle.addEventListener('mousedown', makeResizeDrag('bottom'));

        // Move drag: translates both start+end vertically; drops into another column by X
        moveHandle.addEventListener('mousedown', (e: MouseEvent) => {
            e.preventDefault();
            e.stopPropagation();

            const startClientY = e.clientY;
            const startClientX = e.clientX;
            const origStart = block.StartMin;
            const origEnd = block.EndMin;
            const duration = origEnd - origStart;
            let hasMoved = false;

            const grid = col.parentElement!;
            const dayCols = [...grid.querySelectorAll<HTMLElement>('.time-blocks-day-col')];

            const onMove = (ev: MouseEvent) => {
                if (Math.abs(ev.clientY - startClientY) > DRAG_THRESHOLD_PX ||
                    Math.abs(ev.clientX - startClientX) > DRAG_THRESHOLD_PX) {
                    hasMoved = true;
                }
                const deltaY = ev.clientY - startClientY;
                const deltaMins = Math.round((deltaY / HOUR_HEIGHT_PX * 60) / MIN_BLOCK_MINS) * MIN_BLOCK_MINS;
                const newStart = clampMins(origStart + deltaMins);
                const newEnd = Math.min(GRID_END_HOUR * 60, newStart + duration);
                card.style.top = `${minsToY(newStart)}px`;
                card.style.height = `${Math.max(18, minsToY(newEnd) - minsToY(newStart))}px`;
                const timeEl = card.querySelector<HTMLSpanElement>('.time-block-time');
                if (timeEl) timeEl.textContent = `${minsToLabel(newStart)}–${minsToLabel(newEnd)} `;
            };

            const onUp = async (ev: MouseEvent) => {
                document.removeEventListener('mousemove', onMove);
                document.removeEventListener('mouseup', onUp);
                if (!hasMoved) return;

                // Find target column by drop X
                let targetDayKey = dayKey;
                for (const dc of dayCols) {
                    const r = dc.getBoundingClientRect();
                    if (ev.clientX >= r.left && ev.clientX <= r.right) {
                        targetDayKey = dc.dataset.date ?? dayKey;
                        break;
                    }
                }

                const deltaY = ev.clientY - startClientY;
                const deltaMins = Math.round((deltaY / HOUR_HEIGHT_PX * 60) / MIN_BLOCK_MINS) * MIN_BLOCK_MINS;
                const newStart = clampMins(origStart + deltaMins);
                const newEnd = Math.min(GRID_END_HOUR * 60, newStart + duration);

                try {
                    const dto: UpdateTimeBlockDto = { block_id: block.BlockId, start_min: newStart, end_min: newEnd };
                    if (targetDayKey !== dayKey) dto.date = targetDayKey;
                    await api.Update(dto);

                    block.StartMin = newStart;
                    block.EndMin = newEnd;

                    if (targetDayKey !== dayKey) {
                        block.Date = DateTime.fromISO(targetDayKey);
                        const srcArr = blocksByDate.get(dayKey) ?? [];
                        blocksByDate.set(dayKey, srcArr.filter(b => b.BlockId !== block.BlockId));
                        const dstArr = blocksByDate.get(targetDayKey) ?? [];
                        dstArr.push(block);
                        blocksByDate.set(targetDayKey, dstArr);
                        refreshCol();
                        refreshColMap.get(targetDayKey)?.();
                    } else {
                        refreshCol();
                    }
                } catch {
                    refreshCol();
                }
            };

            document.addEventListener('mousemove', onMove);
            document.addEventListener('mouseup', onUp);
        });

        return card;
    }

    private buildBlockCard(block: TimeBlock, onDelete: (() => void) | (() => Promise<void>)): HTMLElement {
        const card = document.createElement('div');
        card.className = 'time-block-card';
        card.title = `${minsToLabel(block.StartMin)}–${minsToLabel(block.EndMin)} · ${block.Item.Title}`;

        const time = document.createElement('span');
        time.className = 'time-block-time';
        time.textContent = `${minsToLabel(block.StartMin)}–${minsToLabel(block.EndMin)} `;
        card.appendChild(time);

        const title = document.createElement('strong');
        title.className = 'time-block-title';
        title.textContent = block.Item.Title;
        card.appendChild(title);

        if (block.Note) {
            const note = document.createElement('p');
            note.className = 'time-block-note';
            note.textContent = block.Note;
            card.appendChild(note);
        }

        // Title is a button so it sits above the move handle (z-index) and navigates on click
        title.addEventListener('click', (e) => {
            e.stopPropagation();
            getNavigator().openItemById(block.Item.ItemID);
        });

        const del = document.createElement('button');
        del.type = 'button';
        del.className = 'time-block-delete-btn';
        del.title = 'Delete';
        del.innerHTML = '&times;';
        del.addEventListener('click', (e) => {
            e.stopPropagation();
            void Promise.resolve(onDelete());
        });
        card.appendChild(del);

        return card;
    }

    private openCreateModal(
        day: DateTime,
        startMins: number,
        endMins: number,
        blocksByDate: Map<string, TimeBlock[]>,
        dayKey: string,
        refreshCol: () => void,
    ): void {
        const overlay = document.createElement('div');
        overlay.className = 'time-block-modal-overlay';

        const modal = document.createElement('div');
        modal.className = 'time-block-modal';

        const title = document.createElement('h3');
        title.className = 'time-block-modal-title';
        title.textContent = `New block · ${day.toFormat('EEE d LLL')}`;
        modal.appendChild(title);

        // Time range row
        const timeRow = document.createElement('div');
        timeRow.className = 'time-block-modal-row';

        const startInput = document.createElement('input');
        startInput.type = 'time';
        startInput.className = 'time-block-modal-time';
        startInput.value = `${Math.floor(startMins / 60).toString().padStart(2, '0')}:${(startMins % 60).toString().padStart(2, '0')}`;

        const sep = document.createElement('span');
        sep.textContent = '–';

        const endInput = document.createElement('input');
        endInput.type = 'time';
        endInput.className = 'time-block-modal-time';
        endInput.value = `${Math.floor(endMins / 60).toString().padStart(2, '0')}:${(endMins % 60).toString().padStart(2, '0')}`;

        timeRow.append(startInput, sep, endInput);
        modal.appendChild(timeRow);

        // Item search
        const searchLabel = document.createElement('label');
        searchLabel.className = 'time-block-modal-label';
        searchLabel.textContent = 'Item';
        modal.appendChild(searchLabel);

        const search = new ItemSearchInline();
        search.placeholder = 'Search for an item…';
        modal.appendChild(search);

        // Note
        const noteLabel = document.createElement('label');
        noteLabel.className = 'time-block-modal-label';
        noteLabel.textContent = 'Note (optional)';
        modal.appendChild(noteLabel);

        const noteInput = document.createElement('input');
        noteInput.type = 'text';
        noteInput.className = 'time-block-modal-note';
        noteInput.placeholder = 'Add a note…';
        modal.appendChild(noteInput);

        const errorEl = document.createElement('p');
        errorEl.className = 'time-block-modal-error';
        errorEl.hidden = true;
        modal.appendChild(errorEl);

        // Buttons
        const btnRow = document.createElement('div');
        btnRow.className = 'time-block-modal-btns';

        const saveBtn = document.createElement('button');
        saveBtn.type = 'button';
        saveBtn.className = 'time-block-modal-save';
        saveBtn.textContent = 'Create';

        const cancelBtn = document.createElement('button');
        cancelBtn.type = 'button';
        cancelBtn.className = 'time-block-modal-cancel';
        cancelBtn.textContent = 'Cancel';

        btnRow.append(saveBtn, cancelBtn);
        modal.appendChild(btnRow);
        overlay.appendChild(modal);
        document.body.appendChild(overlay);

        const close = () => overlay.remove();
        cancelBtn.addEventListener('click', close);
        overlay.addEventListener('click', (e) => { if (e.target === overlay) close(); });

        const parseTime = (input: HTMLInputElement): number => {
            const [h, m] = input.value.split(':').map(Number);
            return (h ?? 0) * 60 + (m ?? 0);
        };

        saveBtn.addEventListener('click', async () => {
            const item: Item | null = search.value;
            if (!item) {
                errorEl.textContent = 'Please select an item.';
                errorEl.hidden = false;
                return;
            }
            const sMin = parseTime(startInput);
            const eMin = parseTime(endInput);
            if (eMin <= sMin) {
                errorEl.textContent = 'End time must be after start time.';
                errorEl.hidden = false;
                return;
            }
            errorEl.hidden = true;
            saveBtn.disabled = true;
            saveBtn.textContent = 'Saving…';
            try {
                const note = noteInput.value.trim();
                const block = await api.Create({
                    item_id: item.ItemID,
                    date: dayKey,
                    start_min: sMin,
                    end_min: eMin,
                    ...(note ? { note } : {}),
                });
                const arr = blocksByDate.get(dayKey) ?? [];
                arr.push(block);
                blocksByDate.set(dayKey, arr);
                refreshCol();
                close();
            } catch {
                saveBtn.disabled = false;
                saveBtn.textContent = 'Create';
                errorEl.textContent = 'Failed to save. Please try again.';
                errorEl.hidden = false;
            }
        });

        requestAnimationFrame(() => search.focus());
    }

    private errorCard(message: string, actionLabel: string, onAction: () => void): HTMLElement {
        const div = document.createElement('div');
        div.className = 'planner-error-card';
        const p = document.createElement('p');
        p.textContent = message;
        div.appendChild(p);
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.textContent = actionLabel;
        btn.addEventListener('click', onAction);
        div.appendChild(btn);
        return div;
    }
}

if (!customElements.get('time-blocks-screen')) {
    customElements.define('time-blocks-screen', TimeBlocksScreen);
}
