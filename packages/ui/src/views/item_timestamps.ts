import type { Item } from '@zealot/domain/src/item';

/** An item's created or updated time in Unix seconds; 0 when unknown. */
export function timestampOf(item: Item, field: 'created' | 'updated'): number {
    return field === 'created' ? item.CreatedAt : item.UpdatedAt;
}

/** A Unix-seconds timestamp as a local `YYYY-MM-DD` date; empty when unknown. */
export function formatTimestamp(seconds: number): string {
    if (!(seconds > 0)) return '';
    const date = new Date(seconds * 1000);
    const pad = (n: number): string => String(n).padStart(2, '0');
    return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}
