import type { AttributeKind } from '@zealot/domain/src/attribute';
import type { Item } from '@zealot/domain/src/item';
import { getComparableAttributeValue } from './attribute_value_input';
import { timestampOf } from './item_timestamps';
import type { ItemTableColumn, SortDirection } from './item_table_types';

export function keyForColumn(column: ItemTableColumn): string {
    if (column.kind === 'attribute') return `attr:${column.attributeKey}`;
    if (column.kind === 'timestamp') return `timestamp:${column.field}`;
    return column.kind;
}

export function sortItems(
    items: Item[],
    sortColumnKey: string | null,
    sortDirection: SortDirection,
    columns: ItemTableColumn[],
    attributeKinds: Record<string, AttributeKind>,
): Item[] {
    const result = [...items];
    if (!sortColumnKey || !sortDirection) return result;

    const column = columns.find((col) => keyForColumn(col) === sortColumnKey);
    if (!column) return result;

    result.sort((left, right) => {
        const leftValue = sortValueForItem(left, column, attributeKinds);
        const rightValue = sortValueForItem(right, column, attributeKinds);

        const leftMissing = leftValue == null || leftValue === '';
        const rightMissing = rightValue == null || rightValue === '';

        if (leftMissing && rightMissing) return 0;
        if (leftMissing) return 1;
        if (rightMissing) return -1;

        if (typeof leftValue === 'number' && typeof rightValue === 'number') {
            return leftValue - rightValue;
        }

        return String(leftValue).localeCompare(String(rightValue), undefined, { sensitivity: 'base' });
    });

    if (sortDirection === 'desc') result.reverse();
    return result;
}

function sortValueForItem(
    item: Item,
    column: ItemTableColumn,
    attributeKinds: Record<string, AttributeKind>,
): string | number | null {
    if (column.kind === 'title') return item.Title.toLowerCase();

    if (column.kind === 'types') {
        const typeNames = item.Types.map((t) => t.Name).join('\u0000').toLowerCase();
        return typeNames === '' ? null : typeNames;
    }

    if (column.kind === 'timestamp') {
        const seconds = timestampOf(item, column.field);
        return seconds > 0 ? seconds : null;
    }

    return getComparableAttributeValue(
        item.Attributes[column.attributeKey],
        attributeKinds[column.attributeKey],
    );
}

export function toggleSort(
    currentKey: string | null,
    currentDir: SortDirection,
    column: ItemTableColumn,
): { key: string | null; dir: SortDirection } {
    const key = keyForColumn(column);
    if (currentKey !== key) return { key, dir: 'asc' };
    if (currentDir === 'asc') return { key, dir: 'desc' };
    if (currentDir === 'desc') return { key: null, dir: null };
    return { key, dir: 'asc' };
}
