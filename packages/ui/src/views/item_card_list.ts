import { getNavigator } from '@websoil/engine';
import type { Item } from '@zealot/domain/src/item';

const STATUS_ORDER = ['Working', 'Specify', 'To Do', 'Complete', 'Hold', 'Rejected', 'Blocked'];

function buildCard(item: Item, showStatus: boolean): HTMLElement {
    const card = document.createElement('button');
    card.type = 'button';
    card.className = 'item-card';
    card.addEventListener('click', () => {
        getNavigator().openItemById(item.ItemID);
    });

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

function buildGroup(label: string, items: Item[], showStatus: boolean): HTMLElement {
    const group = document.createElement('div');
    group.className = 'item-card-group';

    const heading = document.createElement('div');
    heading.className = 'item-card-group__label';
    heading.textContent = label;
    group.appendChild(heading);

    for (const item of items) {
        group.appendChild(buildCard(item, showStatus));
    }

    return group;
}

function groupItems(items: Item[]): Array<{ label: string; items: Item[]; showStatus: boolean }> {
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

    const result: Array<{ label: string; items: Item[]; showStatus: boolean }> = [];

    // Status groups in defined order first, then any remaining unknown statuses
    for (const status of STATUS_ORDER) {
        const bucket = statusBuckets.get(status);
        if (bucket) result.push({ label: status, items: bucket, showStatus: false });
    }
    for (const [status, bucket] of statusBuckets) {
        if (!STATUS_ORDER.includes(status)) {
            result.push({ label: status, items: bucket, showStatus: false });
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
    options: { grouped?: boolean } = {},
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
        for (const { label, items: bucketItems, showStatus } of groupItems(items)) {
            list.appendChild(buildGroup(label, bucketItems, showStatus));
        }
    } else {
        for (const item of items) {
            list.appendChild(buildCard(item, true));
        }
    }

    return list;
}
