import { getNavigator } from '@websoil/engine';
import type { AttributeKind } from '@zealot/domain/src/attribute';
import type { Item } from '@zealot/domain/src/item';
import { loadAttributeKinds } from './attribute_value_input';
import { buildCreateRow } from './item_table_create_row';
import { saveAttribute, saveTitle, saveTypes, createItem } from './item_table_save';
import { keyForColumn, sortItems, toggleSort } from './item_table_sort';
import ChipsInput from './chips_input';
import {
    createAttributeValueInput,
} from './attribute_value_input';
import { isBlankAttributeValue } from './attribute_value_input';

export type { ItemTableColumn, ItemTableCreateRowConfig, ItemTableViewConfig } from './item_table_types';
import type { CreateDraftState, SortDirection } from './item_table_types';
import type { ItemTableColumn, ItemTableViewConfig } from './item_table_types';

export class ItemTableView extends HTMLElement {
    private _attributeKinds: Record<string, AttributeKind> = {};
    private _config: ItemTableViewConfig | null = null;
    private _createDraft: CreateDraftState = this._newCreateDraft();
    private _createError: string | null = null;
    private _creating = false;
    private _items: Item[] = [];
    private _sortColumnKey: string | null = null;
    private _sortDirection: SortDirection = null;

    connectedCallback(): void {
        if (this._config) {
            this._render();
            void this._ensureKinds();
        }
    }

    init(config: ItemTableViewConfig): this {
        this._config = config;
        this._items = [...config.items];
        this._createError = null;
        this._creating = false;
        this._createDraft = this._newCreateDraft();
        this._sortColumnKey = null;
        this._sortDirection = null;
        this._render();
        void this._ensureKinds();
        return this;
    }

    private async _ensureKinds(): Promise<void> {
        this._attributeKinds = await loadAttributeKinds();
        this._render();
    }

    private _render(): void {
        if (!this._config) {
            this.innerHTML = '';
            return;
        }

        this.className = 'item-table-view';
        this.innerHTML = '';

        const shell = document.createElement('div');
        shell.className = 'item-table-shell';

        if (this._createError) {
            const error = document.createElement('p');
            error.className = 'tool-error item-table-error';
            error.textContent = this._createError;
            shell.appendChild(error);
        }

        const table = document.createElement('table');
        table.className = 'item-table';

        const thead = document.createElement('thead');
        const headerRow = document.createElement('tr');
        for (const column of this._config.columns) {
            const header = document.createElement('th');
            header.textContent = this._labelForColumn(column);

            if (column.sortable !== false) {
                header.classList.add('item-table-sortable');
                if (this._sortColumnKey === keyForColumn(column)) {
                    header.textContent += this._sortDirection === 'asc' ? ' ^' : this._sortDirection === 'desc' ? ' v' : '';
                }
                header.addEventListener('click', () => {
                    const next = toggleSort(this._sortColumnKey, this._sortDirection, column);
                    this._sortColumnKey = next.key;
                    this._sortDirection = next.dir;
                    this._render();
                });
            }

            headerRow.appendChild(header);
        }

        if (this._config.createRow?.enabled) {
            const actionHeader = document.createElement('th');
            actionHeader.className = 'item-table-actions-header';
            headerRow.appendChild(actionHeader);
        }

        thead.appendChild(headerRow);
        table.appendChild(thead);

        const tbody = document.createElement('tbody');
        if (this._config.createRow?.enabled) {
            tbody.appendChild(buildCreateRow(
                this._config,
                this._createDraft,
                this._creating,
                this._attributeKinds,
                () => { void this._submitCreateRow(); },
            ));
        }

        const items = sortItems(this._items, this._sortColumnKey, this._sortDirection, this._config.columns, this._attributeKinds);
        if (items.length === 0) {
            const row = document.createElement('tr');
            const cell = document.createElement('td');
            cell.colSpan = this._columnCount();
            cell.className = 'tool-muted item-table-empty';
            cell.textContent = this._config.emptyMessage ?? 'No items.';
            row.appendChild(cell);
            tbody.appendChild(row);
        } else {
            items.forEach((item) => {
                tbody.appendChild(this._buildItemRow(item));
            });
        }

        table.appendChild(tbody);
        shell.appendChild(table);
        this.appendChild(shell);
    }

    private _buildItemRow(item: Item): HTMLTableRowElement {
        const row = document.createElement('tr');
        row.addEventListener('click', (event) => {
            if (this._isInteractiveTarget(event.target)) return;
            const openItem = this._config?.onOpenItem ?? ((target: Item) => {
                getNavigator().openItemById(target.ItemID);
            });
            openItem(item);
        });

        for (const column of this._config!.columns) {
            const cell = document.createElement('td');
            cell.appendChild(this._buildItemCell(item, column));
            row.appendChild(cell);
        }

        if (this._config!.createRow?.enabled) {
            const actionCell = document.createElement('td');
            actionCell.className = 'item-table-actions';
            row.appendChild(actionCell);
        }

        return row;
    }

    private _buildItemCell(item: Item, column: ItemTableColumn): HTMLElement {
        if (column.kind === 'title') {
            if (column.editable === false) return this._readOnlyCell(item.DisplayTitle);

            const input = document.createElement('input');
            input.type = 'text';
            input.value = item.Title;
            input.addEventListener('keydown', (event) => {
                if (event.key === 'Enter') { event.preventDefault(); input.blur(); }
            });
            input.addEventListener('blur', () => {
                void saveTitle(item, input).then((result) => {
                    if (result === 'saved' && this._sortColumnKey === 'title') this._render();
                });
            });
            return input;
        }

        if (column.kind === 'types') {
            if (column.editable === false) {
                return this._readOnlyCell(item.Types.map((t) => t.Name).join(', '));
            }

            const chips = new ChipsInput();
            chips.value = item.Types.map((t) => t.Name);
            chips.OnClickItem = (name) => { getNavigator().openType(name); };
            const syncTypes = (): void => {
                void saveTypes(item, chips.value.slice()).then((result) => {
                    if (result === 'failed' || (result === 'saved' && this._sortColumnKey === 'types')) {
                        this._render();
                    }
                });
            };
            chips.addEventListener('chips-add', syncTypes);
            chips.addEventListener('chips-remove', syncTypes);
            return chips;
        }

        const key = column.attributeKey;
        const kind = this._attributeKinds[key];

        if (column.editable === false) {
            return this._readOnlyCell(this._formatAttributeValue(item.Attributes[key]));
        }

        const controlOptions: Parameters<typeof createAttributeValueInput>[0] = {
            allowEmpty: true,
            attributeKey: key,
            onCommit: async (value) => {
                const result = await saveAttribute(item, key, value, kind);
                if (result === 'saved') {
                    item.Attributes[key] = value;
                    if (this._sortColumnKey === keyForColumn(column)) {
                        this._render();
                    }
                    return;
                }
                if (result === 'removed') {
                    delete item.Attributes[key];
                }
                this._render();
            },
            value: item.Attributes[key],
        };
        if (kind) controlOptions.kind = kind;

        return createAttributeValueInput(controlOptions).element;
    }

    private async _submitCreateRow(): Promise<void> {
        const createConfig = this._config?.createRow;
        if (!createConfig?.enabled || this._creating) return;

        const title = this._createDraft.title.trim();
        if (!title) {
            this._createError = 'Title is required.';
            this._render();
            this._focusCreateTitle();
            return;
        }

        const attributes: Record<string, unknown> = { ...(createConfig.defaultAttributes ?? {}) };
        for (const column of this._config!.columns) {
            if (column.kind !== 'attribute' || column.editable === false) continue;
            const kind = this._attributeKinds[column.attributeKey];
            const value = this._createDraft.attributes[column.attributeKey];
            if (!isBlankAttributeValue(value, kind)) {
                attributes[column.attributeKey] = value;
            }
        }

        this._creating = true;
        this._createError = null;
        this._render();

        try {
            const created = await createItem({
                attributes,
                title,
                types: this._createDraft.types,
                ...(createConfig.contextItemId != null ? { contextItemId: createConfig.contextItemId } : {}),
                ...(createConfig.relationship != null ? { relationship: createConfig.relationship } : {}),
            });
            this._items.unshift(created);
            this._createDraft = this._newCreateDraft();
            this._createError = null;
            createConfig.onSuccess?.(created);
        } catch (error) {
            this._createError = (error as Error).message ?? 'Failed to create item.';
        } finally {
            this._creating = false;
            this._render();
            this._focusCreateTitle();
        }
    }

    private _focusCreateTitle(): void {
        window.requestAnimationFrame(() => {
            const input = this.querySelector('[data-item-table-create-title="true"]') as HTMLInputElement | null;
            input?.focus();
        });
    }

    private _columnCount(): number {
        return this._config!.columns.length + (this._config!.createRow?.enabled ? 1 : 0);
    }

    private _labelForColumn(column: ItemTableColumn): string {
        if (column.label) return column.label;
        if (column.kind === 'title') return 'Title';
        if (column.kind === 'types') return 'Type';
        return column.attributeKey;
    }

    private _readOnlyCell(value: string): HTMLElement {
        const span = document.createElement('span');
        span.textContent = value;
        return span;
    }

    private _formatAttributeValue(value: unknown): string {
        if (Array.isArray(value)) return value.map((entry) => String(entry)).join(', ');
        if (typeof value === 'boolean') return value ? 'True' : 'False';
        if (value == null) return '';
        return String(value);
    }

    private _isInteractiveTarget(target: EventTarget | null): boolean {
        if (!(target instanceof Element)) return false;
        return target.closest('button, input, select, textarea, a, chips-input, item-picker-input, item-chips-input') !== null;
    }

    private _newCreateDraft(): CreateDraftState {
        return { attributes: {}, title: '', types: [] };
    }
}

if (!customElements.get('item-table-view')) {
    customElements.define('item-table-view', ItemTableView);
}
