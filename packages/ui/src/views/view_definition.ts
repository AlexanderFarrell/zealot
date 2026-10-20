import type { AttributeKind } from '@zealot/domain/src/attribute';

/** Operators the item query understands for an attribute filter. */
export const FILTER_OPS = ['eq', 'ne', 'gt', 'lt', 'gte', 'lte', 'ilike', 'exists', 'missing'] as const;
export type FilterOp = typeof FILTER_OPS[number];

export const OP_LABELS: Record<FilterOp, string> = {
    eq: 'is',
    ne: 'is not',
    gt: 'is after / above',
    lt: 'is before / below',
    gte: 'is on or after / at least',
    lte: 'is on or before / at most',
    ilike: 'contains',
    exists: 'is set',
    missing: 'is not set',
};

/** Relative date tokens the server resolves when the view runs, in the account time zone. */
export const DATE_TOKENS: Array<{ value: string; label: string }> = [
    { value: '$today', label: 'today' },
    { value: '$today-7d', label: 'today − 7 days' },
    { value: '$today+7d', label: 'today + 7 days' },
    { value: '$this_week', label: 'this week' },
    { value: '$this_week-1w', label: 'last week' },
    { value: '$this_week+1w', label: 'next week' },
    { value: '$this_month', label: 'this month' },
    { value: '$this_year', label: 'this year' },
];

/** Keys that are not attributes but can be filtered and sorted on. */
export const TIMESTAMP_KEYS = ['created', 'updated'] as const;

export interface FilterDef {
    key: string;
    op: FilterOp;
    value?: unknown;
    list_mode?: 'any' | 'all' | 'none';
    [extra: string]: unknown;
}

export interface QueryDef {
    types?: string[];
    type_mode?: 'any' | 'all';
    include_archived?: boolean;
    under?: number;
    linked_from?: number;
    filters?: FilterDef[];
    any_of?: FilterDef[][];
    sort?: { key: string; direction?: 'asc' | 'desc' };
    limit?: number;
    [extra: string]: unknown;
}

export interface ViewDef {
    query: QueryDef;
    group_by?: string;
    columns?: string[];
    display?: 'cards' | 'table';
    [extra: string]: unknown;
}

export type KindClass = 'text' | 'number' | 'date' | 'boolean' | 'enum' | 'list' | 'item';

/** How the builder treats an attribute key's values; unknown keys behave as text. */
export function classOfKind(key: string, kind?: AttributeKind): KindClass {
    if ((TIMESTAMP_KEYS as readonly string[]).includes(key)) return 'date';
    switch (kind?.BaseType) {
        case 'integer':
        case 'decimal':
            return 'number';
        case 'date':
        case 'week':
            return 'date';
        case 'boolean':
            return 'boolean';
        case 'dropdown':
            return 'enum';
        case 'list':
            return 'list';
        case 'item':
            return 'item';
        default:
            return 'text';
    }
}

/** Operators that make sense for a kind, in the order shown. */
export function opsForClass(kind: KindClass): FilterOp[] {
    switch (kind) {
        case 'number':
        case 'date':
            return ['eq', 'ne', 'gt', 'lt', 'gte', 'lte', 'exists', 'missing'];
        case 'text':
            return ['eq', 'ne', 'ilike', 'exists', 'missing'];
        default:
            return ['eq', 'ne', 'exists', 'missing'];
    }
}

export function opNeedsValue(op: FilterOp): boolean {
    return op !== 'exists' && op !== 'missing';
}

export function starterDefinition(): ViewDef {
    return { query: { types: ['Ticket'] }, display: 'cards' };
}

/** Content of a new Saved View: a line of prose and the starter definition in its fenced block. */
export function starterContent(): string {
    return `What this view shows.\n\n\`\`\`zealot-view\n${JSON.stringify(starterDefinition(), null, 2)}\n\`\`\`\n`;
}

export type ParseResult = { ok: true; def: ViewDef } | { ok: false; error: string };

export function parseDefinitionText(text: string): ParseResult {
    let value: unknown;
    try {
        value = JSON.parse(text);
    } catch (error) {
        return { ok: false, error: (error as Error).message };
    }
    if (value === null || typeof value !== 'object' || Array.isArray(value)) {
        return { ok: false, error: 'A definition is a JSON object' };
    }
    const def = value as ViewDef;
    if (def.query === undefined) def.query = {};
    if (def.query === null || typeof def.query !== 'object' || Array.isArray(def.query)) {
        return { ok: false, error: '"query" must be an object' };
    }
    return { ok: true, def };
}

/** Two-space JSON, as written back into the block. Unknown keys survive because nothing is dropped. */
export function stringifyDefinition(def: ViewDef): string {
    return JSON.stringify(def, null, 2);
}

const QUERY_KEYS = new Set([
    'types', 'type_mode', 'include_archived', 'under', 'linked_from', 'filters', 'any_of', 'sort', 'limit',
]);
const FILTER_KEYS = new Set(['key', 'op', 'value', 'list_mode']);

const isFilter = (value: unknown): boolean => {
    if (value === null || typeof value !== 'object' || Array.isArray(value)) return false;
    const filter = value as Record<string, unknown>;
    if (typeof filter.key !== 'string') return false;
    if (!(FILTER_OPS as readonly string[]).includes(filter.op as string)) return false;
    if (filter.list_mode !== undefined && !['any', 'all', 'none'].includes(filter.list_mode as string)) return false;
    return Object.keys(filter).every((k) => FILTER_KEYS.has(k));
};

/**
 * Why the form cannot show this definition, or null when it can. Top-level keys the form does not
 * know are kept untouched and do not block it; anything inside `query` it cannot show does.
 */
export function unrepresentableReason(def: ViewDef): string | null {
    const query = def.query;
    for (const key of Object.keys(query)) {
        if (!QUERY_KEYS.has(key)) return `The query uses "${key}", which the form does not cover.`;
    }
    if (query.types !== undefined && !(Array.isArray(query.types) && query.types.every((t) => typeof t === 'string'))) {
        return '"types" is not a list of type names.';
    }
    if (query.type_mode !== undefined && query.type_mode !== 'any' && query.type_mode !== 'all') {
        return `"type_mode" ${JSON.stringify(query.type_mode)} is not any or all.`;
    }
    for (const key of ['under', 'linked_from', 'limit'] as const) {
        const value = query[key];
        if (value !== undefined && !Number.isInteger(value)) return `"${key}" is not a whole number.`;
    }
    if (query.include_archived !== undefined && typeof query.include_archived !== 'boolean') {
        return '"include_archived" is not true or false.';
    }
    if (query.filters !== undefined && !(Array.isArray(query.filters) && query.filters.every(isFilter))) {
        return 'A filter uses a field or operator the form does not cover.';
    }
    if (query.any_of !== undefined) {
        const ok = Array.isArray(query.any_of)
            && query.any_of.every((group) => Array.isArray(group) && group.every(isFilter));
        if (!ok) return 'An "any_of" group uses a field or operator the form does not cover.';
    }
    if (query.sort !== undefined) {
        const sort = query.sort as Record<string, unknown> | null;
        const ok = sort !== null && typeof sort === 'object' && typeof sort.key === 'string'
            && (sort.direction === undefined || ['asc', 'desc', ''].includes(sort.direction as string))
            && Object.keys(sort).every((k) => k === 'key' || k === 'direction');
        if (!ok) return '"sort" is not a key with an optional direction.';
    }
    if (def.group_by !== undefined && typeof def.group_by !== 'string') return '"group_by" is not an attribute key.';
    if (def.columns !== undefined && !(Array.isArray(def.columns) && def.columns.every((c) => typeof c === 'string'))) {
        return '"columns" is not a list of attribute keys.';
    }
    if (def.display !== undefined && def.display !== 'cards' && def.display !== 'table') {
        return `"display" ${JSON.stringify(def.display)} is not cards or table.`;
    }
    return null;
}

/** A value for a new filter row of the given kind, so the row is valid the moment it is added. */
export function newFilter(key: string, kind: KindClass): FilterDef {
    const op = opsForClass(kind)[0] ?? 'eq';
    return { key, op };
}

/** Drops empty optional parts so the written JSON stays short; never touches unknown keys. */
export function tidy(def: ViewDef): ViewDef {
    const query = def.query;
    if (query.types && query.types.length === 0) delete query.types;
    if (query.type_mode === 'any') delete query.type_mode;
    if (query.include_archived === false) delete query.include_archived;
    if (query.filters && query.filters.length === 0) delete query.filters;
    if (query.any_of) {
        query.any_of = query.any_of.filter((group) => group.length > 0);
        if (query.any_of.length === 0) delete query.any_of;
    }
    if (query.sort && !query.sort.key) delete query.sort;
    if (def.group_by === '') delete def.group_by;
    if (def.columns && def.columns.length === 0) delete def.columns;
    return def;
}
