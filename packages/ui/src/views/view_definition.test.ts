import { describe, expect, it } from 'vitest';
import {
    classOfKind, opsForClass, parseDefinitionText, starterContent, stringifyDefinition, tidy, unrepresentableReason,
} from './view_definition';
import type { ViewDef } from './view_definition';
import type { AttributeKind } from '@zealot/domain/src/attribute';

const parsed = (text: string): ViewDef => {
    const result = parseDefinitionText(text);
    if (!result.ok) throw new Error(result.error);
    return result.def;
};

const AI_QUEUE = `{
  "query": {
    "types": ["Plan", "Ticket"],
    "type_mode": "any",
    "filters": [
      { "key": "Executor", "op": "eq", "value": "LLM" },
      { "key": "Status", "op": "eq", "value": ["To Do", "Working"] }
    ],
    "sort": { "key": "Priority", "direction": "desc" },
    "limit": 100
  },
  "group_by": "Parent",
  "columns": ["Status", "Priority"]
}`;

describe('view definition', () => {
    it('shows a typical view in the form', () => {
        expect(unrepresentableReason(parsed(AI_QUEUE))).toBeNull();
    });

    it('covers any_of groups, exists/missing, relative dates, under and linked_from', () => {
        const def = parsed(JSON.stringify({
            query: {
                under: 12, linked_from: 3, include_archived: true,
                filters: [{ key: 'Date', op: 'missing' }, { key: 'updated', op: 'lt', value: '$today-30d' }],
                any_of: [[{ key: 'Week', op: 'exists' }], [{ key: 'Date', op: 'gte', value: '$this_week' }]],
            },
        }));
        expect(unrepresentableReason(def)).toBeNull();
    });

    it.each([
        ['an unknown query key', { query: { offset: 5 } }],
        ['an unknown operator', { query: { filters: [{ key: 'A', op: 'regex', value: 'x' }] } }],
        ['an extra filter field', { query: { filters: [{ key: 'A', op: 'eq', value: 1, why: 'x' }] } }],
        ['a non-numeric under', { query: { under: 'Home' } }],
        ['a bad display', { query: {}, display: 'pie' }],
        ['a bad any_of group', { query: { any_of: [{ key: 'A' }] } }],
    ])('falls back to JSON for %s', (_name, def) => {
        expect(unrepresentableReason(def as unknown as ViewDef)).not.toBeNull();
    });

    it('keeps top-level keys the form does not know', () => {
        const def = parsed('{"query": {}, "future": {"a": 1}}');
        expect(unrepresentableReason(def)).toBeNull();
        expect(JSON.parse(stringifyDefinition(tidy(def)))).toEqual({ query: {}, future: { a: 1 } });
    });

    it('pretty-prints with two spaces', () => {
        expect(stringifyDefinition({ query: { limit: 5 } })).toBe('{\n  "query": {\n    "limit": 5\n  }\n}');
    });

    it('tidy drops only empty optional parts', () => {
        const def = tidy({
            query: { types: [], type_mode: 'any', include_archived: false, filters: [], any_of: [[]] },
            columns: [], group_by: '',
        });
        expect(def).toEqual({ query: {} });
    });

    it('rejects text that is not a definition object', () => {
        expect(parseDefinitionText('[1]').ok).toBe(false);
        expect(parseDefinitionText('{oops').ok).toBe(false);
        expect(parseDefinitionText('{"query": 3}').ok).toBe(false);
    });

    it('filters operators by attribute kind', () => {
        const kind = (BaseType: string) => ({ BaseType }) as AttributeKind;
        expect(opsForClass(classOfKind('Priority', kind('integer')))).toContain('gt');
        expect(opsForClass(classOfKind('Status', kind('dropdown')))).not.toContain('gt');
        expect(opsForClass(classOfKind('Description', kind('text')))).toContain('ilike');
        expect(classOfKind('created')).toBe('date');
        expect(classOfKind('Anything')).toBe('text');
    });

    it('starts a new view with a block the editor and server both read', () => {
        const content = starterContent();
        const match = /```zealot-view\n([\s\S]*?)\n```/.exec(content);
        expect(match).not.toBeNull();
        const def = parsed(match![1]!);
        expect(unrepresentableReason(def)).toBeNull();
        expect(def.query.types).toEqual(['Ticket']);
    });
});
