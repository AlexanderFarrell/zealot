import type { AttributeBaseType } from '@zealot/domain/src/attribute';

/** Free-text default, parsed per base type; the server validates it against the kind. */
export function parseDefaultInput(baseType: AttributeBaseType, raw: string): unknown {
    switch (baseType) {
        case 'integer':
        case 'decimal':
            return Number(raw);
        case 'boolean':
            return raw === 'true';
        case 'list':
            return raw.split(',').map((part) => part.trim()).filter((part) => part !== '');
        default:
            return raw;
    }
}

export function formatDefaultInput(value: unknown): string {
    if (value === undefined || value === null) return '';
    return Array.isArray(value) ? value.join(', ') : String(value);
}
