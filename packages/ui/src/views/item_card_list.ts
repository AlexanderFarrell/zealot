import { getNavigator, registerDropZone, registerContextMenu, Popups } from '@websoil/engine';
import { AttributeAPI } from '@zealot/api/src/attribute';
import { ItemAPI } from '@zealot/api/src/item';
import type { Item } from '@zealot/domain/src/item';
import { ConfirmDialog } from '../common/confirm_dialog';

const attrApi = new AttributeAPI('/api');
const itemApi = new ItemAPI('/api');

const STATUS_ORDER = ['Working', 'Specify', 'To Do', 'Complete', 'Hold', 'Rejected', 'Blocked'];

function buildCard(item: Item, showStatus: boolean, onDrop?: () => void): HTMLElement {
    const card = document.createElement('button');
    card.type = 'button';
    card.className = 'item-card';
    card.draggable = true;

    card.addEventListener('click', () => {
        getNavigator().openItemById(item.ItemID);
    });

    card.addEventListener('dragstart', (e) => {
        e.dataTransfer?.setData('text/plain', String(item.ItemID));
        if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move';
    });

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

    registerContextMenu(card, () => [
        { label: 'Open', onClick: () => getNavigator().openItemById(item.ItemID) },
        { label: 'Open in New Tab', onClick: () => window.open(`/item/${encodeURIComponent(item.Title)}`, '_blank') },
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
    title.textContent = item.DisplayTitle;
    header.appendChild(title);

    if (showStatus) {
        const status = item.Attributes['Status'];
        if (status != null && status !== '') {
            const badge = document.createElement('div');
            badge.className = 'item-card__status';
            badge.textContent = String(status);
            header.appendChild(badge);
        }
    }

    card.appendChild(header);

    const typeNames = item.Types.filter((t) => !t.IsSystem).map((t) => t.Name);
    if (typeNames.length > 0) {
        const typeEl = document.createElement('div');
        typeEl.className = 'item-card__type';
        typeEl.textContent = typeNames.join(' · ');
        card.appendChild(typeEl);
    }

    return card;
}

function buildGroup(label: string, items: Item[], showStatus: boolean, statusValue?: string, onDrop?: () => void): HTMLElement {
    const group = document.createElement('div');
    group.className = 'item-card-group';

    const heading = document.createElement('div');
    heading.className = 'item-card-group__label';
    heading.textContent = label;
    group.appendChild(heading);

    if (statusValue !== undefined) {
        registerDropZone({
            element: heading,
            onDragOver: (e: DragEvent) => {
                if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
                heading.classList.add('item-card-group__label--drop-target');
            },
            onDrop: (draggedItemId: number) => {
                void attrApi.set_value(draggedItemId, 'Status', statusValue).then(() => onDrop?.());
            },
            onDragLeave: () => {
                heading.classList.remove('item-card-group__label--drop-target');
            },
        });
    }

    for (const item of items) {
        group.appendChild(buildCard(item, showStatus, onDrop));
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
        if (!STATUS_ORDER.includes(status)) {
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

    return result;
}

export function buildItemCardList(
    items: Item[],
    emptyMessage: string,
    options: { grouped?: boolean; onDrop?: () => void } = {},
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

    if (options.grouped) {
        for (const { label, items: bucketItems, showStatus, statusValue } of groupItems(items)) {
            list.appendChild(buildGroup(label, bucketItems, showStatus, statusValue, options.onDrop));
        }
    } else {
        for (const item of items) {
            list.appendChild(buildCard(item, true, options.onDrop));
        }
    }

    return list;
}
