/**
 * Daily planner scoring, grouping and carry-over (Z277).
 *
 * Pure functions over `Item`s so the rules can be tested without a DOM. Nothing
 * here names a user's own attributes: which attributes give points, which
 * dropdown groups the list and which types move to the header all come from
 * Planner settings.
 */
import type { DateTime } from 'luxon';
import type { Item } from '@zealot/domain/src/item';

/** Days before the shown day that an unfinished item keeps being carried over. */
export const CARRY_OVER_DAYS = 14;

/** Statuses that never carry over: finished, decided against, or paused on purpose. */
export const CARRY_OVER_EXCLUDED_STATUSES = ['Complete', 'Rejected', 'Retired', 'Hold'];

/** Status is a read-only system dropdown, so its group order is fixed (the planner's order before Z277). */
export const STATUS_GROUP_ORDER = ['Working', 'Specify', 'To Do', 'On Going', 'Hold', 'Blocked', 'Complete'];

/** Status values that always group last, after any values not in `STATUS_GROUP_ORDER`. */
export const STATUS_GROUP_BOTTOM = ['Rejected', 'Retired'];

/** Group key for items that have no value for the grouping attribute. */
export const NO_VALUE_GROUP = '__none__';

/** Group key for items without a Status (notes, journal entries): not work, so always last. */
export const OTHER_GROUP = '__other__';

/** The dropdown attribute the list is grouped by, with its values in order. */
export interface GroupBy {
    key: string;
    /** The dropdown's values, in the attribute's own order. */
    values: string[];
}

/** One group of cards: `key` is the attribute value (or `NO_VALUE_GROUP`), `label` what is shown. */
export interface ItemGroup {
    key: string;
    label: string;
    items: Item[];
}

/** Reads a numeric attribute, treating missing, blank or non-numeric values as `fallback`. */
export function numericAttribute(item: Item, key: string, fallback: number): number {
    const raw: unknown = item.Attributes[key];
    if (raw === null || raw === undefined || raw === '') return fallback;
    const value = typeof raw === 'number' ? raw : Number(raw);
    return Number.isFinite(value) ? value : fallback;
}

/** What an item is worth: `100 × ` the product of `pointsFrom`, each missing value counting as 1. */
export function itemPoints(item: Item, pointsFrom: string[]): number {
    return pointsFrom.reduce((product, key) => product * numericAttribute(item, key, 1), 100);
}

/** A copy of `items` sorted by points, highest first; ties, and every item when points are off, keep their order. */
export function sortByPoints(items: Item[], pointsFrom: string[]): Item[] {
    if (pointsFrom.length === 0) return [...items];
    return [...items].sort((a, b) => itemPoints(b, pointsFrom) - itemPoints(a, pointsFrom));
}

/** True when the item has a Status, so it is work: it can carry over and it counts toward the score. */
export function isWorkItem(item: Item): boolean {
    const status: unknown = item.Attributes['Status'];
    return status !== null && status !== undefined && status !== '';
}

/** True when the item has one of `headerTypes`, so it shows as a header link instead of a card. */
export function isHeaderItem(item: Item, headerTypes: string[]): boolean {
    return item.Types.some((type) => headerTypes.includes(type.Name));
}

/** The order groups appear in: the dropdown's values, or the fixed Status order. */
function knownOrder(groupBy: GroupBy): { top: string[]; bottom: string[] } {
    if (groupBy.key === 'Status') return { top: STATUS_GROUP_ORDER, bottom: STATUS_GROUP_BOTTOM };
    return { top: groupBy.values, bottom: [] };
}

/** The group key an item falls in: its value, or `NO_VALUE_GROUP` when it has none. */
export function groupKeyOf(item: Item, groupKey: string): string {
    const raw: unknown = item.Attributes[groupKey];
    if (raw === null || raw === undefined || raw === '') return NO_VALUE_GROUP;
    return String(raw);
}

/**
 * Groups items by a dropdown: known values in order, then values not in the
 * list (first seen first), then Status's bottom values, then "No <Attribute>",
 * then "Other" for items without a Status. Empty groups are left out and each
 * group is sorted by points. With no grouping, work is one unlabelled group
 * and "Other" follows it.
 */
export function groupItems(items: Item[], groupBy: GroupBy | null, pointsFrom: string[]): ItemGroup[] {
    const work = items.filter(isWorkItem);
    const other = items.filter((item) => !isWorkItem(item));
    const groups = groupWork(work, groupBy, pointsFrom);
    if (other.length > 0) groups.push({ key: OTHER_GROUP, label: 'Other', items: sortByPoints(other, pointsFrom) });
    return groups;
}

function groupWork(items: Item[], groupBy: GroupBy | null, pointsFrom: string[]): ItemGroup[] {
    if (!groupBy) {
        return items.length === 0 ? [] : [{ key: '', label: '', items: sortByPoints(items, pointsFrom) }];
    }

    const buckets = new Map<string, Item[]>();
    for (const item of items) {
        const key = groupKeyOf(item, groupBy.key);
        const bucket = buckets.get(key);
        if (bucket) bucket.push(item);
        else buckets.set(key, [item]);
    }

    const { top, bottom } = knownOrder(groupBy);
    const unknown = [...buckets.keys()].filter((key) => key !== NO_VALUE_GROUP && !top.includes(key) && !bottom.includes(key));
    const order = [...top, ...unknown, ...bottom, NO_VALUE_GROUP];

    const groups: ItemGroup[] = [];
    for (const key of order) {
        const bucket = buckets.get(key);
        if (!bucket) continue;
        groups.push({
            key,
            label: key === NO_VALUE_GROUP ? `No ${groupBy.key}` : key,
            items: sortByPoints(bucket, pointsFrom),
        });
    }
    return groups;
}

/** The ISO dates bounding the carry-over window: `from` is 14 days back, `to` the day before. */
export function carryOverWindow(day: DateTime): { from: string; to: string } {
    const iso = (d: DateTime): string => d.toISODate() ?? d.toFormat('yyyy-MM-dd');
    return {
        from: iso(day.minus({ days: CARRY_OVER_DAYS })),
        to: iso(day.minus({ days: 1 })),
    };
}

/**
 * True when an item slipped from a recent day: its Date is within the 14 days
 * before `day`, it has a Status that is not finished or held, and it is not a
 * header item.
 */
export function isCarriedOver(item: Item, day: DateTime, headerTypes: string[]): boolean {
    const date: unknown = item.Attributes['Date'];
    if (typeof date !== 'string') return false;
    const { from, to } = carryOverWindow(day);
    // ISO dates compare correctly as strings.
    if (date < from || date > to) return false;
    if (!isWorkItem(item)) return false;
    const status = String(item.Attributes['Status']);
    if (CARRY_OVER_EXCLUDED_STATUSES.includes(status)) return false;
    return !isHeaderItem(item, headerTypes);
}

/** Today's earned and planned points. */
export interface DayScore {
    earned: number;
    planned: number;
}

/**
 * Earned counts the day's Complete items; planned counts the day's work items
 * plus the carried-over ones. Items without a Status count toward neither.
 * Callers pass lists without header items.
 */
export function dayScore(dayItems: Item[], carriedOver: Item[], pointsFrom: string[]): DayScore {
    const earned = dayItems
        .filter((item) => item.Attributes['Status'] === 'Complete')
        .reduce((sum, item) => sum + itemPoints(item, pointsFrom), 0);
    const planned = [...dayItems, ...carriedOver]
        .filter(isWorkItem)
        .reduce((sum, item) => sum + itemPoints(item, pointsFrom), 0);
    return { earned, planned };
}

/** Formats points for display with thousands separators (`2,400`). */
export function formatPoints(points: number): string {
    return Math.round(points).toLocaleString('en-GB');
}

/**
 * Short chip labels for the points attributes: the name when it is three
 * letters or fewer (`AP`), otherwise its first letter (`Priority` → `P`),
 * falling back to the full name when two first letters would clash.
 */
export function chipLabels(pointsFrom: string[]): Record<string, string> {
    // Callers pass any list of number attributes (points or card fields).
    const short = (key: string): string => (key.length <= 3 ? key : key.charAt(0).toUpperCase());
    const labels: Record<string, string> = {};
    for (const key of pointsFrom) {
        const label = short(key);
        const clash = pointsFrom.some((other) => other !== key && short(other) === label);
        labels[key] = clash ? key : label;
    }
    return labels;
}
