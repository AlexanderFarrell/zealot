/**
 * Chips on item cards (Z277): what the item is worth, and the fields chosen in
 * Planner settings ("Edit on cards") as small chips that edit inline when
 * tapped: numbers as an input, dropdowns as a pick list.
 */
import { Popups } from '@websoil/engine';
import { AttributeAPI } from '@zealot/api/src/attribute';
import type { Item } from '@zealot/domain/src/item';
import { chipLabels, formatPoints, itemPoints } from '../screens/daily_planner_model';

const attrApi = new AttributeAPI('/api');

/** A field that edits inline on cards. */
export interface CardField {
    key: string;
    kind: 'number' | 'dropdown';
    /** A dropdown's values, in order. */
    values?: string[];
}

/** Builds the points chip (when points are on) plus an editable chip per card field. */
export function buildCardChips(item: Item, pointsFrom: string[], fields: CardField[], onChange: () => void): HTMLElement {
    const wrap = document.createElement('span');
    wrap.className = 'item-card__points';

    if (pointsFrom.length > 0) {
        const points = document.createElement('span');
        points.className = 'item-card__chip item-card__chip--points';
        points.textContent = `${formatPoints(itemPoints(item, pointsFrom))} pts`;
        points.title = ['100', ...pointsFrom].join(' × ');
        wrap.appendChild(points);
    }

    const labels = chipLabels(fields.filter((f) => f.kind === 'number').map((f) => f.key));
    for (const field of fields) {
        wrap.appendChild(field.kind === 'number'
            ? buildEditableChip(item, field.key, labels[field.key] ?? field.key, onChange)
            : buildDropdownChip(item, field.key, field.values ?? [], onChange));
    }
    return wrap;
}

/** Saves a field optimistically, putting the old value back if the server refuses. */
function saveField(item: Item, key: string, value: unknown, onChange: () => void): void {
    const previous: unknown = item.Attributes[key];
    const clear = value === null;
    if (clear) delete item.Attributes[key];
    else item.Attributes[key] = value;
    onChange();
    const request = clear ? attrApi.remove(item.ItemID, key) : attrApi.set_value(item.ItemID, key, value);
    request.catch(() => {
        if (previous === undefined) delete item.Attributes[key];
        else item.Attributes[key] = previous;
        Popups.add(`Could not save ${key}`);
        onChange();
    });
}

/** Keeps a press on a chip or its editor from opening the item or starting a drag. */
function shield(el: HTMLElement): void {
    for (const type of ['click', 'pointerdown', 'mousedown', 'dragstart'] as const) {
        el.addEventListener(type, (e) => e.stopPropagation());
    }
}

/** A chip showing the dropdown's value; tapping it opens a pick list that saves on change. */
function buildDropdownChip(item: Item, key: string, values: string[], onChange: () => void): HTMLElement {
    const chip = document.createElement('span');
    chip.className = 'item-card__chip item-card__chip--edit';
    chip.setAttribute('role', 'button');
    chip.tabIndex = 0;
    chip.dataset['key'] = key;
    chip.title = `Edit ${key}`;

    const paint = (): void => {
        const raw: unknown = item.Attributes[key];
        const missing = raw === null || raw === undefined || raw === '';
        chip.textContent = missing ? `${key} –` : String(raw);
        chip.classList.toggle('item-card__chip--missing', missing);
    };
    paint();

    const startEdit = (e: Event): void => {
        e.stopPropagation();
        e.preventDefault();
        if (chip.querySelector('select')) return;
        const current: unknown = item.Attributes[key];
        const select = document.createElement('select');
        select.className = 'item-card__chip-select';
        select.setAttribute('aria-label', key);
        const options = [...values];
        if (typeof current === 'string' && current !== '' && !options.includes(current)) options.push(current);
        const blank = document.createElement('option');
        blank.value = '';
        blank.textContent = `No ${key}`;
        select.appendChild(blank);
        for (const value of options) {
            const option = document.createElement('option');
            option.value = value;
            option.textContent = value;
            select.appendChild(option);
        }
        select.value = current === null || current === undefined ? '' : String(current);
        shield(select);
        let done = false;
        const close = (): void => {
            if (done) return;
            done = true;
            paint();
        };
        select.addEventListener('change', () => {
            const next = select.value;
            close();
            if (next === String(current ?? '')) return;
            saveField(item, key, next === '' ? null : next, onChange);
        });
        select.addEventListener('keydown', (ev) => {
            ev.stopPropagation();
            if (ev.key === 'Escape') { ev.preventDefault(); close(); }
        });
        select.addEventListener('blur', close);
        chip.textContent = '';
        chip.appendChild(select);
        select.focus();
        // Open the list straight away where the browser allows it.
        (select as HTMLSelectElement & { showPicker?: () => void }).showPicker?.();
    };
    chip.addEventListener('click', startEdit);
    chip.addEventListener('keydown', (e) => { if (e.key === 'Enter' || e.key === ' ') startEdit(e); });
    chip.addEventListener('pointerdown', (e) => e.stopPropagation());
    return chip;
}

/** A chip showing `P8` or `AP3`; tapping it swaps in a number input that saves on Enter or blur. */
function buildEditableChip(item: Item, key: string, label: string, onChange: () => void): HTMLElement {
    const chip = document.createElement('span');
    chip.className = 'item-card__chip item-card__chip--edit';
    chip.setAttribute('role', 'button');
    chip.tabIndex = 0;
    chip.dataset['key'] = key;
    chip.title = `Edit ${key}`;

    const paint = (): void => {
        const raw: unknown = item.Attributes[key];
        const missing = raw === null || raw === undefined || raw === '';
        chip.textContent = `${label}${missing ? '–' : String(raw)}`;
        chip.classList.toggle('item-card__chip--missing', missing);
    };
    paint();

    const startEdit = (e: Event): void => {
        e.stopPropagation();
        e.preventDefault();
        if (chip.querySelector('input')) return;
        openEditor(chip, item, key, label, onChange, paint);
    };
    chip.addEventListener('click', startEdit);
    chip.addEventListener('keydown', (e) => { if (e.key === 'Enter' || e.key === ' ') startEdit(e); });
    // Keep the card from starting a drag when the chip is pressed.
    chip.addEventListener('pointerdown', (e) => e.stopPropagation());
    return chip;
}

/** Replaces the chip's text with a number input and saves the new value. */
function openEditor(chip: HTMLElement, item: Item, key: string, label: string, onChange: () => void, paint: () => void): void {
    const previous: unknown = item.Attributes[key];
    const input = document.createElement('input');
    input.type = 'number';
    input.inputMode = 'numeric';
    input.min = '0';
    input.step = '1';
    input.className = 'item-card__chip-input';
    input.setAttribute('aria-label', key);
    input.value = previous === null || previous === undefined ? '' : String(previous);

    chip.textContent = label;
    chip.appendChild(input);
    input.focus();
    input.select();

    let done = false;
    const finish = (save: boolean): void => {
        if (done) return;
        done = true;
        if (!save) {
            paint();
            return;
        }
        const text = input.value.trim();
        if (text === '' || text === String(previous ?? '')) {
            paint();
            return;
        }
        const value = Number(text);
        if (!Number.isInteger(value) || value < 0) {
            Popups.add(`${key} must be a whole number`);
            paint();
            return;
        }
        saveField(item, key, value, onChange);
    };

    shield(input);
    input.addEventListener('keydown', (e) => {
        e.stopPropagation();
        if (e.key === 'Enter') { e.preventDefault(); finish(true); }
        if (e.key === 'Escape') { e.preventDefault(); finish(false); }
    });
    input.addEventListener('keyup', (e) => e.preventDefault());
    input.addEventListener('blur', () => finish(true));
}
