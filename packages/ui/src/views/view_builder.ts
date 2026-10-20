import { ItemTypeAPI } from '@zealot/api/src/item_type';
import { ViewAPI, ViewError } from '@zealot/api/src/view';
import type { AttributeKind } from '@zealot/domain/src/attribute';
import { loadAttributeKinds } from './attribute_value_input';
import ChipsInput from './chips_input';
import { ItemPickerInput } from './item_picker_input';
import { ViewResultBox } from './view_result_box';
import {
    DATE_TOKENS, FILTER_OPS, OP_LABELS, TIMESTAMP_KEYS, classOfKind, newFilter, opNeedsValue, opsForClass,
    parseDefinitionText, stringifyDefinition, tidy, unrepresentableReason,
} from './view_definition';
import type { FilterDef, FilterOp, KindClass, ViewDef } from './view_definition';

const viewApi = new ViewAPI('/api');
const typeApi = new ItemTypeAPI('/api');

const PREVIEW_DEBOUNCE_MS = 500;

export interface ViewBuilderConfig {
    /** The block's current JSON text. */
    text: string;
    title: string;
    /** `text` is the new block text, or null when nothing changed. */
    onDone: (text: string | null) => void;
    onCancel: () => void;
}

function el<K extends keyof HTMLElementTagNameMap>(
    tag: K, className?: string, text?: string,
): HTMLElementTagNameMap[K] {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (text !== undefined) node.textContent = text;
    return node;
}

function labelled(label: string, control: HTMLElement, className = 'view-builder__field'): HTMLElement {
    const wrap = el('label', className);
    wrap.append(el('span', 'view-builder__label', label), control);
    return wrap;
}

function select(options: Array<[string, string]>, value: string, onChange: (value: string) => void): HTMLSelectElement {
    const node = el('select');
    for (const [v, text] of options) {
        const option = el('option', undefined, text);
        option.value = v;
        node.appendChild(option);
    }
    node.value = value;
    node.addEventListener('change', () => onChange(node.value));
    return node;
}

/**
 * The Edit view surface for a Saved View: a form over the definition, the raw JSON, and a live
 * preview of the unsaved definition. The form and the JSON edit one object, so keys the form does
 * not know survive. Done reports the new block text; the host writes it through the editor.
 */
export class ViewBuilder extends HTMLElement {
    private _config!: ViewBuilderConfig;
    private _def: ViewDef | null = null;
    private _jsonText = '';
    private _jsonError: string | null = null;
    private _reason: string | null = null;
    private _formEdited = false;
    private _jsonEdited = false;
    private _kinds: Record<string, AttributeKind> = {};
    private _typeNames: string[] = [];
    private _previewTimer: ReturnType<typeof setTimeout> | null = null;
    private _form = el('div', 'view-builder__form');
    private _jsonArea = el('textarea', 'view-builder__json');
    private _jsonNote = el('p', 'tool-error view-builder__json-error');
    private _today = el('span', 'tool-muted view-builder__today');
    private _preview = new ViewResultBox();

    init(config: ViewBuilderConfig): this {
        this._config = config;
        this._jsonText = config.text;
        this._readJson();
        this._build();
        void this._loadChoices();
        return this;
    }

    private async _loadChoices(): Promise<void> {
        const [kinds, types] = await Promise.all([
            loadAttributeKinds(),
            typeApi.Types.Get().catch(() => ({})),
        ]);
        this._kinds = kinds;
        this._typeNames = Object.keys(types).sort();
        this._renderForm();
    }

    // Model

    /** Parses the JSON text into the working definition; the form shows only when it can. */
    private _readJson(): void {
        const result = parseDefinitionText(this._jsonText);
        if (!result.ok) {
            this._jsonError = result.error;
            this._reason = null;
            return;
        }
        this._jsonError = null;
        this._def = result.def;
        this._reason = unrepresentableReason(result.def);
    }

    /** A form edit changed the definition: refresh the JSON text, then the preview. */
    private _formChanged(): void {
        if (!this._def) return;
        this._formEdited = true;
        this._jsonText = stringifyDefinition(tidy(structuredClone(this._def)));
        this._jsonArea.value = this._jsonText;
        this._jsonError = null;
        this._syncJsonNote();
        this._schedulePreview();
    }

    private _jsonChanged(): void {
        this._jsonEdited = true;
        this._jsonText = this._jsonArea.value;
        this._readJson();
        this._syncJsonNote();
        // The form follows the JSON when it parses; while it does not, the form waits.
        if (!this._jsonError) this._renderForm();
        this._schedulePreview();
    }

    private _syncJsonNote(): void {
        this._jsonNote.textContent = this._jsonError ? `JSON: ${this._jsonError}` : '';
        this._jsonNote.hidden = !this._jsonError;
    }

    private get _query() {
        return this._def!.query;
    }

    // Layout

    private _build(): void {
        this.className = 'view-builder';
        this.innerHTML = '';

        const header = el('div', 'view-builder__header');
        header.append(el('strong', undefined, `Edit view: ${this._config.title}`));
        const actions = el('div', 'view-builder__actions');
        const cancel = el('button', 'view-builder__cancel', 'Cancel');
        cancel.type = 'button';
        cancel.addEventListener('click', () => this._config.onCancel());
        const done = el('button', 'view-builder__done', 'Done');
        done.type = 'button';
        done.addEventListener('click', () => this._done());
        actions.append(cancel, done);
        header.appendChild(actions);

        const body = el('div', 'view-builder__body');
        const editor = el('div', 'view-builder__editor');
        editor.appendChild(this._form);

        const json = el('details', 'view-builder__json-wrap');
        json.appendChild(el('summary', undefined, 'JSON'));
        this._jsonArea.value = this._jsonText;
        this._jsonArea.spellcheck = false;
        this._jsonArea.rows = 12;
        this._jsonArea.addEventListener('input', () => this._jsonChanged());
        this._syncJsonNote();
        json.append(this._jsonArea, this._jsonNote);
        // Collapsed unless the form cannot show the definition.
        json.open = this._jsonError !== null || this._reason !== null;
        editor.appendChild(json);

        const previewWrap = el('div', 'view-builder__preview');
        previewWrap.append(el('strong', undefined, 'Preview'), this._today);
        this._preview.init({
            title: this._config.title,
            display: this._def?.display === 'table' ? 'table' : 'cards',
            load: (page) => {
                if (!this._def || this._jsonError) {
                    throw new ViewError('invalid', `Invalid definition: ${this._jsonError ?? 'unreadable'}`);
                }
                return viewApi.RunDefinition(this._def, page);
            },
            onDisplayChange: (display) => {
                if (!this._def) return;
                this._def.display = display;
                this._formChanged();
            },
            paged: true,
        });
        previewWrap.appendChild(this._preview);

        body.append(editor, previewWrap);
        this.append(header, body);
        this._renderForm();
        void this._refreshToday();
    }

    private async _refreshToday(): Promise<void> {
        if (!this._def || this._jsonError) { this._today.textContent = ''; return; }
        try {
            const report = await viewApi.Validate(this._def);
            this._today.textContent = `Today resolves to ${report.today} (${report.time_zone})`;
        } catch {
            this._today.textContent = '';
        }
    }

    private _schedulePreview(): void {
        if (this._previewTimer) clearTimeout(this._previewTimer);
        this._previewTimer = setTimeout(() => {
            void this._preview.refresh();
            void this._refreshToday();
        }, PREVIEW_DEBOUNCE_MS);
    }

    disconnectedCallback(): void {
        if (this._previewTimer) clearTimeout(this._previewTimer);
    }

    private _done(): void {
        if (this._jsonError) {
            this._jsonNote.hidden = false;
            this._jsonArea.focus();
            return;
        }
        if (!this._formEdited && !this._jsonEdited) {
            this._config.onDone(null);
            return;
        }
        // A form edit already wrote tidy JSON into the text; a JSON edit is the user's own, so
        // it is only re-indented.
        const result = parseDefinitionText(this._jsonText);
        this._config.onDone(result.ok ? stringifyDefinition(result.def) : this._jsonText);
    }

    // Form

    private _renderForm(): void {
        this._form.innerHTML = '';
        if (!this._def) {
            this._form.appendChild(el('p', 'tool-muted', 'Fix the JSON below to use the form.'));
            return;
        }
        if (this._reason) {
            this._form.appendChild(el(
                'p', 'tool-muted view-builder__note',
                `${this._reason} Edit the JSON; the preview still runs, and nothing is changed until you do.`,
            ));
            return;
        }
        this._form.append(
            this._typesSection(),
            this._filtersSection(),
            this._anyOfSection(),
            this._scopeSection(),
            this._resultSection(),
        );
    }

    private _section(title: string): HTMLElement {
        const section = el('section', 'view-builder__section');
        section.appendChild(el('h4', undefined, title));
        return section;
    }

    private _typesSection(): HTMLElement {
        const section = this._section('Types');
        const chips = new ChipsInput();
        chips.value = [...(this._query.types ?? [])];
        const sync = (): void => {
            this._query.types = chips.value.slice();
            this._formChanged();
            this._renderForm();
        };
        chips.addEventListener('chips-add', sync);
        chips.addEventListener('chips-remove', sync);
        section.appendChild(chips);
        if (this._typeNames.length > 0) {
            const list = el('datalist');
            list.id = 'view-builder-types';
            for (const name of this._typeNames) list.appendChild(el('option')).setAttribute('value', name);
            section.appendChild(list);
            queueMicrotask(() => chips.querySelector('input')?.setAttribute('list', list.id));
        }
        if ((this._query.types?.length ?? 0) > 1) {
            section.appendChild(labelled('Match', select(
                [['any', 'any of these types'], ['all', 'all of these types']],
                this._query.type_mode ?? 'any',
                (value) => { this._query.type_mode = value as 'any' | 'all'; this._formChanged(); },
            )));
        }
        return section;
    }

    private _filtersSection(): HTMLElement {
        const section = this._section('Filters');
        const filters = (this._query.filters ??= []);
        section.appendChild(this._filterList(filters));
        return section;
    }

    private _anyOfSection(): HTMLElement {
        const groups = this._query.any_of ?? [];
        const section = this._section('And at least one of');
        // Hidden until a group exists; the button is the only way to start one.
        groups.forEach((group, index) => {
            const box = el('div', 'view-builder__group');
            box.appendChild(this._filterList(group));
            const remove = el('button', 'view-builder__remove', 'Remove group');
            remove.type = 'button';
            remove.addEventListener('click', () => {
                groups.splice(index, 1);
                if (groups.length === 0) delete this._query.any_of;
                this._formChanged();
                this._renderForm();
            });
            box.appendChild(remove);
            section.appendChild(box);
        });
        const add = el('button', 'view-builder__add', groups.length === 0 ? 'Add an "at least one of" group' : 'Add group');
        add.type = 'button';
        add.addEventListener('click', () => {
            (this._query.any_of ??= []).push([newFilter('Status', 'enum')]);
            this._formChanged();
            this._renderForm();
        });
        section.appendChild(add);
        return section;
    }

    private _attributeKeys(): string[] {
        return [...Object.keys(this._kinds)].sort();
    }

    private _filterList(filters: FilterDef[]): HTMLElement {
        const list = el('div', 'view-builder__filters');
        filters.forEach((filter, index) => {
            list.appendChild(this._filterRow(filter, () => {
                filters.splice(index, 1);
                this._formChanged();
                this._renderForm();
            }));
        });
        const add = el('button', 'view-builder__add', 'Add filter');
        add.type = 'button';
        add.addEventListener('click', () => {
            filters.push(newFilter('Status', classOfKind('Status', this._kinds['Status'])));
            this._formChanged();
            this._renderForm();
        });
        list.appendChild(add);
        return list;
    }

    private _filterRow(filter: FilterDef, onRemove: () => void): HTMLElement {
        const row = el('div', 'view-builder__filter');
        const kind = this._kinds[filter.key];
        const cls = classOfKind(filter.key, kind);
        const ops = opsForClass(cls);
        // Keep an operator the definition already uses even when the kind would not offer it.
        const shown: FilterOp[] = ops.includes(filter.op) ? ops : [filter.op, ...ops];

        const key = el('input', 'view-builder__key');
        key.type = 'text';
        key.value = filter.key;
        key.setAttribute('list', 'view-builder-keys');
        key.setAttribute('aria-label', 'Attribute');
        key.addEventListener('change', () => {
            filter.key = key.value.trim();
            const next = opsForClass(classOfKind(filter.key, this._kinds[filter.key]));
            if (!next.includes(filter.op)) filter.op = next[0] ?? 'eq';
            delete filter.value;
            delete filter.list_mode;
            this._formChanged();
            this._renderForm();
        });

        const op = select(
            FILTER_OPS.filter((o) => shown.includes(o)).map((o) => [o, OP_LABELS[o]] as [string, string]),
            filter.op,
            (value) => {
                filter.op = value as FilterOp;
                if (!opNeedsValue(filter.op)) { delete filter.value; delete filter.list_mode; }
                this._formChanged();
                this._renderForm();
            },
        );
        op.setAttribute('aria-label', 'Operator');

        row.append(key, op);
        if (opNeedsValue(filter.op)) row.appendChild(this._valueEditor(filter, cls, kind));

        const remove = el('button', 'view-builder__remove', '×');
        remove.type = 'button';
        remove.title = 'Remove filter';
        remove.setAttribute('aria-label', 'Remove filter');
        remove.addEventListener('click', onRemove);
        row.appendChild(remove);
        return row;
    }

    private _valueEditor(filter: FilterDef, cls: KindClass, kind?: AttributeKind): HTMLElement {
        const wrap = el('span', 'view-builder__value');
        const set = (value: unknown): void => {
            if (value === undefined || value === '') delete filter.value;
            else filter.value = value;
            this._formChanged();
        };

        if ((cls === 'enum' || cls === 'list') && (kind?.Config.values?.length ?? 0) > 0) {
            const choices = kind!.Config.values!;
            const current = new Set<string>(
                Array.isArray(filter.value) ? filter.value.map(String) : filter.value != null ? [String(filter.value)] : [],
            );
            for (const choice of choices) {
                const box = el('label', 'view-builder__choice');
                const input = el('input');
                input.type = 'checkbox';
                input.checked = current.has(choice);
                input.addEventListener('change', () => {
                    if (input.checked) current.add(choice); else current.delete(choice);
                    const picked = choices.filter((c) => current.has(c));
                    // One value is a plain match; several mean "is any of".
                    set(picked.length === 0 ? undefined : picked.length === 1 ? picked[0] : picked);
                    this._renderForm();
                });
                box.append(input, document.createTextNode(choice));
                wrap.appendChild(box);
            }
            if (Array.isArray(filter.value) || cls === 'list') {
                wrap.appendChild(select(
                    [['any', 'any'], ['all', 'all'], ['none', 'none']],
                    filter.list_mode ?? 'any',
                    (value) => {
                        if (value === 'any') delete filter.list_mode; else filter.list_mode = value as 'all' | 'none';
                        this._formChanged();
                    },
                ));
            }
            return wrap;
        }

        if (cls === 'date') {
            const text = typeof filter.value === 'string' ? filter.value : '';
            const isToken = text.startsWith('$');
            const date = el('input');
            date.type = 'date';
            date.value = isToken ? '' : text;
            date.setAttribute('aria-label', 'Date');
            const token = select(
                [['', 'or relative…'], ...DATE_TOKENS.map((t) => [t.value, t.label] as [string, string])],
                isToken ? text : '',
                (value) => { date.value = ''; set(value || undefined); },
            );
            date.addEventListener('change', () => { token.value = ''; set(date.value || undefined); });
            wrap.append(date, token);
            return wrap;
        }

        if (cls === 'item') {
            const picker = new ItemPickerInput();
            picker.value = typeof filter.value === 'number' ? filter.value : null;
            picker.OnChange = (id) => set(id ?? undefined);
            wrap.appendChild(picker);
            return wrap;
        }

        if (cls === 'boolean') {
            wrap.appendChild(select(
                [['', '—'], ['true', 'true'], ['false', 'false']],
                filter.value === true ? 'true' : filter.value === false ? 'false' : '',
                (value) => set(value === '' ? undefined : value === 'true'),
            ));
            return wrap;
        }

        const input = el('input');
        input.setAttribute('aria-label', 'Value');
        if (cls === 'number') {
            input.type = 'number';
            input.value = filter.value == null ? '' : String(filter.value);
            input.addEventListener('input', () => set(input.value === '' ? undefined : Number(input.value)));
        } else {
            input.type = 'text';
            input.value = filter.value == null ? '' : Array.isArray(filter.value) ? filter.value.join(', ') : String(filter.value);
            input.addEventListener('input', () => set(input.value));
        }
        wrap.appendChild(input);
        return wrap;
    }

    private _scopeSection(): HTMLElement {
        const section = this._section('Scope');
        const row = el('div', 'view-builder__row');

        const under = new ItemPickerInput();
        under.value = this._query.under ?? null;
        under.OnChange = (id) => {
            if (id === null) delete this._query.under; else this._query.under = id;
            this._formChanged();
        };
        row.appendChild(labelled('Under', under));

        const linked = new ItemPickerInput();
        linked.value = this._query.linked_from ?? null;
        linked.OnChange = (id) => {
            if (id === null) delete this._query.linked_from; else this._query.linked_from = id;
            this._formChanged();
        };
        row.appendChild(labelled('Linked from', linked));

        const archived = el('input');
        archived.type = 'checkbox';
        archived.checked = this._query.include_archived === true;
        archived.addEventListener('change', () => { this._query.include_archived = archived.checked; this._formChanged(); });
        row.appendChild(labelled('Include archived', archived, 'view-builder__field view-builder__field--inline'));

        section.appendChild(row);
        return section;
    }

    private _resultSection(): HTMLElement {
        const section = this._section('Result');
        const row = el('div', 'view-builder__row');
        const keys = this._attributeKeys();

        const sortKeys: Array<[string, string]> = [
            ['', 'Default order'], ['title', 'title'],
            ...TIMESTAMP_KEYS.map((k) => [k, k] as [string, string]),
            ...keys.map((k) => [k, k] as [string, string]),
        ];
        const sortKey = this._query.sort?.key ?? '';
        // A sort key the attribute list no longer has stays selectable.
        if (sortKey && !sortKeys.some(([v]) => v === sortKey)) sortKeys.push([sortKey, sortKey]);
        const sort = select(sortKeys, sortKey, (value) => {
            if (!value) delete this._query.sort;
            else this._query.sort = { key: value, direction: this._query.sort?.direction ?? 'asc' };
            this._formChanged();
            this._renderForm();
        });
        const direction = select(
            [['asc', 'ascending'], ['desc', 'descending']],
            this._query.sort?.direction === 'desc' ? 'desc' : 'asc',
            (value) => {
                if (this._query.sort) this._query.sort.direction = value as 'asc' | 'desc';
                this._formChanged();
            },
        );
        direction.disabled = !this._query.sort;
        row.append(labelled('Sort', sort), direction);

        const limit = el('input');
        limit.type = 'number';
        limit.min = '1';
        limit.max = '100';
        limit.value = this._query.limit == null ? '' : String(this._query.limit);
        limit.addEventListener('input', () => {
            if (limit.value === '') delete this._query.limit; else this._query.limit = Math.trunc(Number(limit.value));
            this._formChanged();
        });
        row.appendChild(labelled('Limit', limit));

        const groupKeys: Array<[string, string]> = [['', 'No grouping'], ...keys.map((k) => [k, k] as [string, string])];
        const current = this._def!.group_by ?? '';
        if (current && !groupKeys.some(([v]) => v === current)) groupKeys.push([current, current]);
        row.appendChild(labelled('Group by', select(groupKeys, current, (value) => {
            if (!value) delete this._def!.group_by; else this._def!.group_by = value;
            this._formChanged();
        })));

        section.appendChild(row);

        const columns = new ChipsInput();
        columns.value = [...(this._def!.columns ?? [])];
        const sync = (): void => {
            this._def!.columns = columns.value.slice();
            this._formChanged();
        };
        columns.addEventListener('chips-add', sync);
        columns.addEventListener('chips-remove', sync);
        section.appendChild(labelled('Table columns', columns));

        const datalist = el('datalist');
        datalist.id = 'view-builder-keys';
        for (const key of [...TIMESTAMP_KEYS, ...keys]) datalist.appendChild(el('option')).setAttribute('value', key);
        section.appendChild(datalist);
        return section;
    }
}

if (!customElements.get('view-builder')) customElements.define('view-builder', ViewBuilder);
