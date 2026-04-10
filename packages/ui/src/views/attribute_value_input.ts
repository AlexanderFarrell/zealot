import { AttributeKindAPI } from '@zealot/api/src/attribute_kind';
import type { AttributeKind } from '@zealot/domain/src/attribute';
import ChipsInput from './chips_input';
import { ItemChipsInput } from './item_chips_input';
import { ItemPickerInput } from './item_picker_input';

const attrKindApi = new AttributeKindAPI('/api');

export interface AttributeValueInputBinding {
    element: HTMLElement;
    focus(): void;
    getValue(): unknown;
}

export interface AttributeValueInputOptions {
    attributeKey: string;
    value: unknown;
    kind?: AttributeKind;
    allowEmpty?: boolean;
    onCommit?: (value: unknown) => void | Promise<void>;
    onValueChange?: (value: unknown) => void;
}

export async function loadAttributeKinds(): Promise<Record<string, AttributeKind>> {
    try {
        return await attrKindApi.Kinds.Get();
    } catch {
        return {};
    }
}

export function isBlankAttributeValue(value: unknown, kind?: AttributeKind): boolean {
    if (value == null) {
        return true;
    }

    if (Array.isArray(value)) {
        return value.length === 0;
    }

    if (typeof value === 'string') {
        return value.trim() === '';
    }

    if (typeof value === 'number') {
        return Number.isNaN(value);
    }

    if (kind?.BaseType === 'boolean') {
        return false;
    }

    return false;
}

export function getComparableAttributeValue(value: unknown, kind?: AttributeKind): string | number | null {
    if (value == null) {
        return null;
    }

    const baseType = kind?.BaseType;

    if (baseType === 'integer' || baseType === 'decimal' || baseType === 'item') {
        if (typeof value === 'number') {
            return Number.isNaN(value) ? null : value;
        }

        if (typeof value === 'string' && value.trim() !== '') {
            const parsed = Number(value);
            return Number.isNaN(parsed) ? null : parsed;
        }

        return null;
    }

    if (baseType === 'date') {
        if (typeof value !== 'string' || value.trim() === '') {
            return null;
        }

        const parsed = Date.parse(value.substring(0, 10));
        return Number.isNaN(parsed) ? value.toLowerCase() : parsed;
    }

    if (baseType === 'week') {
        if (typeof value !== 'string' || value.trim() === '') {
            return null;
        }

        const comparable = toComparableWeek(value);
        return comparable ?? value.toLowerCase();
    }

    if (baseType === 'boolean') {
        if (typeof value === 'boolean') {
            return value ? 1 : 0;
        }

        if (value === 'true') return 1;
        if (value === 'false') return 0;
        return null;
    }

    if (Array.isArray(value)) {
        return value.map((entry) => String(entry)).join('\u0000').toLowerCase();
    }

    return String(value).toLowerCase();
}

export function createAttributeValueInput(options: AttributeValueInputOptions): AttributeValueInputBinding {
    const kind = options.kind;
    const baseType = kind?.BaseType;

    if (!baseType || baseType === 'text') {
        const input = document.createElement('input');
        input.type = 'text';
        input.value = options.value != null ? String(options.value) : '';
        return bindTextInput(input, options, () => input.value);
    }

    if (baseType === 'integer') {
        const input = document.createElement('input');
        input.type = 'number';
        input.step = '1';
        if (kind.Config.min != null) input.min = String(kind.Config.min);
        if (kind.Config.max != null) input.max = String(kind.Config.max);
        input.value = options.value != null ? String(options.value) : '';
        return bindTextInput(input, options, () => readNumberValue(input));
    }

    if (baseType === 'decimal') {
        const input = document.createElement('input');
        input.type = 'number';
        input.step = 'any';
        if (kind.Config.min != null) input.min = String(kind.Config.min);
        if (kind.Config.max != null) input.max = String(kind.Config.max);
        input.value = options.value != null ? String(options.value) : '';
        return bindTextInput(input, options, () => readNumberValue(input));
    }

    if (baseType === 'date') {
        const input = document.createElement('input');
        input.type = 'date';
        input.value = options.value != null ? String(options.value).substring(0, 10) : '';
        return bindChangeInput(input, options, () => readStringValue(input));
    }

    if (baseType === 'week') {
        const input = document.createElement('input');
        input.type = 'week';
        input.value = options.value != null ? String(options.value) : '';
        return bindChangeInput(input, options, () => readStringValue(input));
    }

    if (baseType === 'boolean') {
        const select = document.createElement('select');
        if (options.allowEmpty) {
            appendOption(select, '', '—');
        }
        appendOption(select, 'true', 'True');
        appendOption(select, 'false', 'False');

        if (options.value === true || options.value === 'true' || options.value === 1) {
            select.value = 'true';
        } else if (options.value === false || options.value === 'false' || options.value === 0) {
            select.value = 'false';
        } else if (options.allowEmpty) {
            select.value = '';
        } else {
            select.value = 'false';
        }

        return bindChangeInput(select, options, () => {
            if (select.value === '') {
                return null;
            }
            return select.value === 'true';
        });
    }

    if (baseType === 'dropdown') {
        const select = document.createElement('select');
        appendOption(select, '', '—');
        (kind.Config.values ?? []).forEach((value) => appendOption(select, value, value));
        select.value = options.value != null ? String(options.value) : '';
        return bindChangeInput(select, options, () => readStringValue(select));
    }

    if (baseType === 'list') {
        return createListInput(options);
    }

    if (baseType === 'item') {
        const picker = new ItemPickerInput();
        picker.value = toNullableNumber(options.value);
        picker.OnChange = (itemId) => {
            options.onValueChange?.(itemId);
            if (options.onCommit) {
                void Promise.resolve(options.onCommit(itemId));
            }
        };

        return {
            element: picker,
            focus: () => {
                picker.querySelector('input')?.focus();
            },
            getValue: () => picker.value,
        };
    }

    const fallback = document.createElement('input');
    fallback.type = 'text';
    fallback.value = options.value != null ? String(options.value) : '';
    return bindTextInput(fallback, options, () => fallback.value);
}

function createListInput(options: AttributeValueInputOptions): AttributeValueInputBinding {
    const kind = options.kind!;
    const listType = kind.Config.list_type;

    if (listType === 'item') {
        const chips = new ItemChipsInput();
        chips.value = Array.isArray(options.value) ? options.value.map((value) => Number(value)) : [];
        chips.OnChange = (ids) => {
            options.onValueChange?.(ids);
            if (options.onCommit) {
                void Promise.resolve(options.onCommit(ids));
            }
        };

        return {
            element: chips,
            focus: () => {
                chips.querySelector('input')?.focus();
            },
            getValue: () => chips.value,
        };
    }

    const chips = new ChipsInput() as ChipsInput & { inputType: 'text' | 'number' | 'date' };
    if (listType === 'integer' || listType === 'decimal') {
        chips.inputType = 'number';
    } else if (listType === 'date') {
        chips.inputType = 'date';
    } else {
        chips.inputType = 'text';
    }

    chips.value = Array.isArray(options.value)
        ? options.value.map((value) => String(value))
        : [];

    chips.addEventListener('change', () => {
        const nextValue = readListValue(chips.value, listType);
        options.onValueChange?.(nextValue);
        if (options.onCommit) {
            void Promise.resolve(options.onCommit(nextValue));
        }
    });

    return {
        element: chips,
        focus: () => {
            chips.querySelector('input')?.focus();
        },
        getValue: () => readListValue(chips.value, listType),
    };
}

function bindTextInput(
    input: HTMLInputElement,
    options: AttributeValueInputOptions,
    readValue: () => unknown,
): AttributeValueInputBinding {
    input.addEventListener('input', () => {
        options.onValueChange?.(readValue());
    });

    input.addEventListener('blur', () => {
        if (options.onCommit) {
            void Promise.resolve(options.onCommit(readValue()));
        }
    });

    input.addEventListener('keydown', (event) => {
        if (event.key === 'Enter') {
            event.preventDefault();
            input.blur();
        }
    });

    return {
        element: input,
        focus: () => input.focus(),
        getValue: readValue,
    };
}

function bindChangeInput(
    input: HTMLInputElement | HTMLSelectElement,
    options: AttributeValueInputOptions,
    readValue: () => unknown,
): AttributeValueInputBinding {
    input.addEventListener('change', () => {
        const nextValue = readValue();
        options.onValueChange?.(nextValue);
        if (options.onCommit) {
            void Promise.resolve(options.onCommit(nextValue));
        }
    });

    return {
        element: input,
        focus: () => input.focus(),
        getValue: readValue,
    };
}

function appendOption(select: HTMLSelectElement, value: string, label: string): void {
    const option = document.createElement('option');
    option.value = value;
    option.textContent = label;
    select.appendChild(option);
}

function readNumberValue(input: HTMLInputElement): number | null {
    if (input.value.trim() === '') {
        return null;
    }

    const parsed = Number(input.value);
    return Number.isNaN(parsed) ? null : parsed;
}

function readStringValue(input: HTMLInputElement | HTMLSelectElement): string | null {
    const value = input.value.trim();
    return value === '' ? null : value;
}

function readListValue(values: string[], listType: string | undefined): unknown[] {
    if (listType === 'integer' || listType === 'decimal') {
        return values
            .map((value) => Number(value))
            .filter((value) => !Number.isNaN(value));
    }

    return values.slice();
}

function toNullableNumber(value: unknown): number | null {
    if (typeof value === 'number') {
        return Number.isNaN(value) ? null : value;
    }

    if (typeof value === 'string' && value.trim() !== '') {
        const parsed = Number(value);
        return Number.isNaN(parsed) ? null : parsed;
    }

    return null;
}

function toComparableWeek(value: string): number | null {
    const match = value.trim().match(/^(\d{4})-W(\d{2})$/);
    if (!match) {
        return null;
    }

    return Number(match[1]) * 100 + Number(match[2]);
}
