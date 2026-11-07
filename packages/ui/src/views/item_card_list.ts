import { getNavigator, openInNewTab, registerDropZone, registerContextMenu, Popups } from '@websoil/engine';
import { AttributeAPI } from '@zealot/api/src/attribute';
import { ItemAPI } from '@zealot/api/src/item';
import type { Item } from '@zealot/domain/src/item';
import { ConfirmDialog } from '../common/confirm_dialog';
import { renderItemTitle } from './item_title';
import { getCachedItem } from './item_id_cache';
import { type CardField, buildCardChips } from './item_card_points';

const attrApi = new AttributeAPI('/api');
const itemApi = new ItemAPI('/api');

const STATUS_ORDER = ['Working', 'Specify', 'To Do', 'On Going', 'Hold', 'Blocked', 'Complete'];
const BOTTOM_STATUS_ORDER = ['Rejected', 'Retired'];

/** Optional additions to each card, used by the daily planner (Z277). */
export interface ItemCardExtras {
    /** Attributes multiplied into points; shows a points chip. */
    pointsFrom?: string[];
    /** Fields that edit inline as chips on the card. */
    fields?: CardField[];
    /** False hides the Status select in the card's corner. Default true. */
    statusSelect?: boolean;
    /** Which cards get the points and field chips; default all. */
    showChips?: (item: Item) => boolean;
    /** A small button on the card, such as "→ Today" on carried-over items. */
    action?: { label: string; title?: string; onClick: (item: Item) => void };
}

function buildCard(item: Item, showStatus: boolean, showParent: boolean, onDrop?: () => void, dropZones = true, extras: ItemCardExtras = {}): HTMLElement {
    const card = document.createElement('button');
    card.type = 'button';
    card.className = 'item-card';
    // Screens can style cards by Status (the daily planner does, Z277).
    const cardStatus: unknown = item.Attributes['Status'];
    if (typeof cardStatus === 'string' && cardStatus !== '') card.dataset['status'] = cardStatus;
    card.draggable = true;

    card.addEventListener('click', () => {
        getNavigator().openItemById(item.ItemID);
    });

    card.addEventListener('dragstart', (e) => {
        e.dataTransfer?.setData('text/plain', String(item.ItemID));
        if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move';
    });

    if (dropZones) {
        registerDropZone({
            element: card,
            onDragOver: (e) => {
                if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
                card.classList.add('item-card--drop-target');
            },
            onDrop: (draggedItemId) => {
                if (draggedItemId === item.ItemID) return;
                void attrApi.set_value(draggedItemId, 'Parent', [item.ItemID]).then(() => onDrop?.());
            },
            onDragLeave: () => {
                card.classList.remove('item-card--drop-target');
            },
        });
    }

    registerContextMenu(card, () => [
        { label: 'Open', onClick: () => getNavigator().openItemById(item.ItemID) },
        { label: 'Open in New Tab', onClick: () => openInNewTab(`/item/${encodeURIComponent(item.Title)}`) },
        { label: 'Open in New Window', onClick: () => window.open(`/item/${encodeURIComponent(item.Title)}`, '_blank', 'noopener,noreferrer') },
        { label: 'Copy Link', onClick: () => {
            void navigator.clipboard.writeText(`${window.location.origin}/item/${encodeURIComponent(item.Title)}`);
            Popups.add('Link copied');
        }},
        { separator: true },
        { label: 'Delete', danger: true, onClick: () => {
            void (async () => {
                const ok = await ConfirmDialog.show(`Delete "${item.DisplayTitle}"?`);
                if (!ok) return;
                await itemApi.Delete(item.ItemID);
                Popups.add(`Deleted ${item.DisplayTitle}`);
                onDrop?.();
            })();
        }},
    ]);

    const header = document.createElement('div');
    header.className = 'item-card__header';

    const title = document.createElement('div');
    title.className = 'item-card__title';
    renderItemTitle(title, item);
    header.appendChild(title);

    const status = item.Attributes['Status'];
    if (status != null && status !== '' && extras.statusSelect !== false) {
        const select = document.createElement('select');
        select.className = 'item-card__status-select';
        for (const s of [...STATUS_ORDER, ...BOTTOM_STATUS_ORDER]) {
            const opt = document.createElement('option');
            opt.value = s;
            opt.textContent = s;
            select.appendChild(opt);
        }
        select.value = String(status);
        select.addEventListener('click', (e) => e.stopPropagation());
        select.addEventListener('change', () => {
            const newStatus = select.value;
            item.Attributes['Status'] = newStatus;
            card.dataset['status'] = newStatus;
            void attrApi.set_value(item.ItemID, 'Status', newStatus).then(() => onDrop?.());
        });
        header.appendChild(select);
    }

    card.appendChild(header);

    const chips = extras.showChips?.(item) ?? true;
    const pointsFrom = chips ? extras.pointsFrom ?? [] : [];
    const fields = chips ? extras.fields ?? [] : [];
    if (pointsFrom.length > 0 || fields.length > 0 || extras.action) {
        const meta = document.createElement('div');
        meta.className = 'item-card__meta';
        if (pointsFrom.length > 0 || fields.length > 0) meta.appendChild(buildCardChips(item, pointsFrom, fields, () => onDrop?.()));
        if (extras.action) {
            const { label, title: actionTitle, onClick } = extras.action;
            const action = document.createElement('span');
            action.className = 'item-card__action';
            action.setAttribute('role', 'button');
            action.tabIndex = 0;
            action.textContent = label;
            if (actionTitle) action.title = actionTitle;
            const run = (e: Event): void => {
                e.stopPropagation();
                e.preventDefault();
                onClick(item);
            };
            action.addEventListener('click', run);
            action.addEventListener('keydown', (e) => { if (e.key === 'Enter' || e.key === ' ') run(e); });
            meta.appendChild(action);
        }
        card.appendChild(meta);
    }

    const typeNames = item.Types.filter((t) => !t.IsSystem).map((t) => t.Name);
    const parentLink = showParent ? item.Links.find((l) => l.Relationship === 'parent') : undefined;

    if (typeNames.length > 0 || parentLink) {
        const typeEl = document.createElement('div');
        typeEl.className = 'item-card__type';
        typeEl.textContent = typeNames.join(' · ');
        if (parentLink) {
            const parentEl = document.createElement('span');
            parentEl.className = 'item-card__parent';
            parentEl.textContent = `#${parentLink.OtherItemID}`;
            void getCachedItem(parentLink.OtherItemID).then((p) => {
                parentEl.textContent = p.DisplayTitle;
            }).catch(() => { /* keep ID placeholder */ });
            typeEl.appendChild(parentEl);
        }
        card.appendChild(typeEl);
    }

    return card;
}

/** A caller-ordered group of cards; with `onToggle` its heading collapses and expands it (Z277). */
export interface ItemCardGroup {
    label: string;
    items: Item[];
    collapsed?: boolean;
    onToggle?: (collapsed: boolean) => void;
    /** A card dropped on the heading; the caller sets the group's value on it (Z277). */
    onDropItem?: (itemId: number) => void;
}

/** Makes a group heading a toggle that shows the card count and hides the cards when collapsed. */
function makeGroupCollapsible(group: HTMLElement, heading: HTMLElement, label: string, count: number, collapsed: boolean, onToggle: (collapsed: boolean) => void): void {
    heading.textContent = '';
    const toggle = document.createElement('button');
    toggle.type = 'button';
    toggle.className = 'item-card-group__toggle';
    toggle.textContent = `${label} (${count})`;
    const apply = (isCollapsed: boolean): void => {
        group.classList.toggle('item-card-group--collapsed', isCollapsed);
        toggle.setAttribute('aria-expanded', String(!isCollapsed));
    };
    apply(collapsed);
    toggle.addEventListener('click', () => {
        const next = !group.classList.contains('item-card-group--collapsed');
        apply(next);
        onToggle(next);
    });
    heading.appendChild(toggle);
}

function buildGroup(label: string, items: Item[], showStatus: boolean, showParent: boolean, statusValue?: string, onDrop?: () => void, dropZones = true, extras: ItemCardExtras = {}, collapse?: Pick<ItemCardGroup, 'collapsed' | 'onToggle'>, onDropItem?: (itemId: number) => void): HTMLElement {
    const group = document.createElement('div');
    group.className = 'item-card-group';

    const heading = document.createElement('div');
    heading.className = 'item-card-group__label';
    heading.textContent = label;
    if (collapse?.onToggle) {
        makeGroupCollapsible(group, heading, label, items.length, collapse.collapsed ?? false, collapse.onToggle);
    }
    // An unlabelled group (a list with no grouping) has no heading.
    if (label !== '') group.appendChild(heading);

    const dropHandler = onDropItem ?? (statusValue !== undefined
        ? (draggedItemId: number) => { void attrApi.set_value(draggedItemId, 'Status', statusValue).then(() => onDrop?.()); }
        : undefined);
    if (dropHandler && dropZones && label !== '') {
        registerDropZone({
            element: heading,
            onDragOver: (e: DragEvent) => {
                if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
                heading.classList.add('item-card-group__label--drop-target');
            },
            onDrop: (draggedItemId: number) => {
                heading.classList.remove('item-card-group__label--drop-target');
                dropHandler(draggedItemId);
            },
            onDragLeave: () => {
                heading.classList.remove('item-card-group__label--drop-target');
            },
        });
    }

    for (const item of items) {
        group.appendChild(buildCard(item, showStatus, showParent, onDrop, dropZones, extras));
    }

    return group;
}

function groupItems(items: Item[]): Array<{ label: string; items: Item[]; showStatus: boolean; statusValue?: string }> {
    const statusBuckets = new Map<string, Item[]>();
    const typeBuckets = new Map<string, Item[]>();
    const ungrouped: Item[] = [];

    for (const item of items) {
        const status = item.Attributes['Status'];
        if (status != null && status !== '') {
            const key = String(status);
            if (!statusBuckets.has(key)) statusBuckets.set(key, []);
            statusBuckets.get(key)!.push(item);
            continue;
        }

        const primaryType = item.Types.find((t) => !t.IsSystem);
        if (primaryType) {
            if (!typeBuckets.has(primaryType.Name)) typeBuckets.set(primaryType.Name, []);
            typeBuckets.get(primaryType.Name)!.push(item);
            continue;
        }

        ungrouped.push(item);
    }

    const result: Array<{ label: string; items: Item[]; showStatus: boolean; statusValue?: string }> = [];

    // Status groups in defined order first, then any remaining unknown statuses
    for (const status of STATUS_ORDER) {
        const bucket = statusBuckets.get(status);
        if (bucket) result.push({ label: status, items: bucket, showStatus: false, statusValue: status });
    }
    for (const [status, bucket] of statusBuckets) {
        if (!STATUS_ORDER.includes(status) && !BOTTOM_STATUS_ORDER.includes(status)) {
            result.push({ label: status, items: bucket, showStatus: false, statusValue: status });
        }
    }

    // Type groups
    for (const [typeName, bucket] of typeBuckets) {
        result.push({ label: typeName, items: bucket, showStatus: true });
    }

    // Ungrouped at the end
    if (ungrouped.length > 0) {
        result.push({ label: 'Other', items: ungrouped, showStatus: true });
    }

    // Bottom status groups appear after types and ungrouped
    for (const status of BOTTOM_STATUS_ORDER) {
        const bucket = statusBuckets.get(status);
        if (bucket) result.push({ label: status, items: bucket, showStatus: false, statusValue: status });
    }

    return result;
}

export function buildItemCardList(
    items: Item[],
    emptyMessage: string,
    options: {
        grouped?: boolean;
        showParent?: boolean;
        onDrop?: () => void;
        /** Groups in the caller's order (a Saved View's `groups`), instead of grouping by Status. */
        groups?: ItemCardGroup[];
        /** Card-on-card Parent drops and Status header drops; off for embedded lists. Default on. */
        dropZones?: boolean;
        /** Per-card additions: points chips and a card action. */
        extras?: ItemCardExtras;
    } = {},
): HTMLElement {
    const list = document.createElement('div');
    list.className = 'item-card-list';

    if (items.length === 0) {
        const empty = document.createElement('p');
        empty.className = 'tool-muted item-card-empty';
        empty.textContent = emptyMessage;
        list.appendChild(empty);
        return list;
    }

    const showParent = options.showParent ?? false;
    const dropZones = options.dropZones ?? true;
    const extras = options.extras ?? {};

    if (options.groups) {
        for (const { label, items: bucketItems, collapsed, onToggle, onDropItem } of options.groups) {
            const collapse = onToggle ? { collapsed: collapsed ?? false, onToggle } : undefined;
            list.appendChild(buildGroup(label, bucketItems, true, showParent, undefined, options.onDrop, dropZones, extras, collapse, onDropItem));
        }
    } else if (options.grouped) {
        for (const { label, items: bucketItems, showStatus, statusValue } of groupItems(items)) {
            list.appendChild(buildGroup(label, bucketItems, showStatus, showParent, statusValue, options.onDrop, dropZones, extras));
        }
    } else {
        for (const item of items) {
            list.appendChild(buildCard(item, true, showParent, options.onDrop, dropZones, extras));
        }
    }

    return list;
}
