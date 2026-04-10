import { Popups, getNavigator } from '@websoil/engine';
import { AttributeAPI } from '@zealot/api/src/attribute';
import { ItemAPI } from '@zealot/api/src/item';
import type { AttributeKind } from '@zealot/domain/src/attribute';
import type { Item, ItemRelationship } from '@zealot/domain/src/item';
import { ItemTypeRef } from '@zealot/domain/src/item_type';
import ChipsInput from './chips_input';
import {
    createAttributeValueInput,
    getComparableAttributeValue,
    isBlankAttributeValue,
    loadAttributeKinds,
} from './attribute_value_input';

const itemApi = new ItemAPI('/api');
const attrApi = new AttributeAPI('/api');

export type ItemTableColumn =
    | {
        kind: 'title';
        editable?: boolean;
        label?: string;
        sortable?: boolean;
    }
    | {
        kind: 'types';
        editable?: boolean;
        label?: string;
        sortable?: boolean;
    }
    | {
        attributeKey: string;
        kind: 'attribute';
        editable?: boolean;
        label?: string;
        sortable?: boolean;
    };

export interface ItemTableCreateRowConfig {
    contextItemId?: number;
    enabled: boolean;
    onSuccess?: (item: Item) => void;
    relationship?: ItemRelationship;
    submitLabel?: string;
}

export interface ItemTableViewConfig {
    columns: ItemTableColumn[];
    createRow?: ItemTableCreateRowConfig;
    emptyMessage?: string;
    items: Item[];
}

interface CreateDraftState {
    attributes: Record<string, unknown>;
    title: string;
    types: string[];
}

type SortDirection = 'asc' | 'desc' | null;

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
        const kinds = await loadAttributeKinds();
        this._attributeKinds = kinds;
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
                if (this._sortColumnKey === this._keyForColumn(column)) {
                    header.textContent += this._sortDirection === 'asc' ? ' ^' : this._sortDirection === 'desc' ? ' v' : '';
                }
                header.addEventListener('click', () => {
                    this._toggleSort(column);
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
            tbody.appendChild(this._buildCreateRow());
        }

        const items = this._sortedItems();
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

    private _buildCreateRow(): HTMLTableRowElement {
        const row = document.createElement('tr');
        row.className = 'item-table-create-row';

        for (const column of this._config!.columns) {
            const cell = document.createElement('td');
            cell.appendChild(this._buildCreateCell(column));
            row.appendChild(cell);
        }

        const actionCell = document.createElement('td');
        actionCell.className = 'item-table-actions';

        const submit = document.createElement('button');
        submit.type = 'button';
        submit.textContent = this._creating
            ? 'Creating...'
            : (this._config!.createRow?.submitLabel ?? 'Create');
        submit.disabled = this._creating;
        submit.addEventListener('click', () => {
            void this._submitCreateRow();
        });

        actionCell.appendChild(submit);
        row.appendChild(actionCell);
        return row;
    }

    private _buildCreateCell(column: ItemTableColumn): HTMLElement {
        if (column.kind === 'title') {
            const input = document.createElement('input');
            input.type = 'text';
            input.placeholder = 'New item title';
            input.value = this._createDraft.title;
            input.dataset.itemTableCreateTitle = 'true';
            input.addEventListener('input', () => {
                this._createDraft.title = input.value;
            });
            input.addEventListener('keydown', (event) => {
                if (event.key === 'Enter') {
                    event.preventDefault();
                    void this._submitCreateRow();
                }
            });
            return input;
        }

        if (column.kind === 'types') {
            const chips = new ChipsInput();
            chips.value = this._createDraft.types;
            chips.OnClickItem = (name) => {
                getNavigator().openType(name);
            };
            chips.addEventListener('chips-add', () => {
                this._createDraft.types = chips.value.slice();
            });
            chips.addEventListener('chips-remove', () => {
                this._createDraft.types = chips.value.slice();
            });
            return chips;
        }

        const kind = this._attributeKinds[column.attributeKey];
        const controlOptions: Parameters<typeof createAttributeValueInput>[0] = {
            allowEmpty: true,
            attributeKey: column.attributeKey,
            onValueChange: (value) => {
                if (isBlankAttributeValue(value, kind)) {
                    delete this._createDraft.attributes[column.attributeKey];
                    return;
                }

                this._createDraft.attributes[column.attributeKey] = value;
            },
            value: this._createDraft.attributes[column.attributeKey],
        };
        if (kind) {
            controlOptions.kind = kind;
        }

        const control = createAttributeValueInput(controlOptions);
        return control.element;
    }

    private _buildItemRow(item: Item): HTMLTableRowElement {
        const row = document.createElement('tr');
        row.addEventListener('click', (event) => {
            if (this._isInteractiveTarget(event.target)) {
                return;
            }

            getNavigator().openItemById(item.ItemID);
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
            if (column.editable === false) {
                return this._readOnlyCell(item.DisplayTitle);
            }

            const input = document.createElement('input');
            input.type = 'text';
            input.value = item.Title;
            input.addEventListener('keydown', (event) => {
                if (event.key === 'Enter') {
                    event.preventDefault();
                    input.blur();
                }
            });
            input.addEventListener('blur', () => {
                void this._saveTitle(item, input);
            });
            return input;
        }

        if (column.kind === 'types') {
            if (column.editable === false) {
                return this._readOnlyCell(item.Types.map((typeRef) => typeRef.Name).join(', '));
            }

            const chips = new ChipsInput();
            chips.value = item.Types.map((typeRef) => typeRef.Name);
            chips.OnClickItem = (name) => {
                getNavigator().openType(name);
            };
            const syncTypes = () => {
                void this._saveTypes(item, chips.value.slice());
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
                const result = await this._saveAttribute(item, key, value, kind);
                if (result === 'saved') {
                    item.Attributes[key] = value;
                    if (this._sortColumnKey === this._keyForColumn(column)) {
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
        if (kind) {
            controlOptions.kind = kind;
        }

        const control = createAttributeValueInput(controlOptions);

        return control.element;
    }

    private async _saveTitle(item: Item, input: HTMLInputElement): Promise<void> {
        const nextTitle = input.value.trim();
        const previousTitle = item.Title;

        if (nextTitle === previousTitle) {
            input.value = previousTitle;
            return;
        }

        if (!nextTitle) {
            input.value = previousTitle;
            return;
        }

        try {
            const updated = await itemApi.Update(item.ItemID, {
                item_id: item.ItemID,
                title: nextTitle,
            });
            item.Title = updated.Title;
            if (this._sortColumnKey === 'title') {
                this._render();
            }
        } catch (error) {
            Popups.add_error((error as Error).message ?? 'Failed to save title.');
            input.value = previousTitle;
        }
    }

    private async _saveTypes(item: Item, nextTypes: string[]): Promise<void> {
        const previousTypes = item.Types.map((typeRef) => typeRef.Name);
        const added = nextTypes.filter((name) => !previousTypes.includes(name));
        const removed = previousTypes.filter((name) => !nextTypes.includes(name));

        if (added.length === 0 && removed.length === 0) {
            return;
        }

        try {
            await Promise.all([
                ...added.map((name) => itemApi.AssignType(item.ItemID, name)),
                ...removed.map((name) => itemApi.UnassignType(item.ItemID, name)),
            ]);

            item.Types = nextTypes.map((name) => new ItemTypeRef({
                is_system: false,
                name,
                type_id: -1,
            }));

            if (this._sortColumnKey === 'types') {
                this._render();
            }
        } catch (error) {
            Popups.add_error((error as Error).message ?? 'Failed to update item types.');
            this._render();
        }
    }

    private async _saveAttribute(
        item: Item,
        key: string,
        value: unknown,
        kind: AttributeKind | undefined,
    ): Promise<'saved' | 'removed' | 'failed'> {
        try {
            if (isBlankAttributeValue(value, kind)) {
                await attrApi.remove(item.ItemID, key);
                return 'removed';
            }

            await attrApi.set_value(item.ItemID, key, value);
            return 'saved';
        } catch (error) {
            Popups.add_error((error as Error).message ?? `Failed to save "${key}".`);
            return 'failed';
        }
    }

    private async _submitCreateRow(): Promise<void> {
        const createConfig = this._config?.createRow;
        if (!createConfig?.enabled || this._creating) {
            return;
        }

        const title = this._createDraft.title.trim();
        if (!title) {
            this._createError = 'Title is required.';
            this._render();
            this._focusCreateTitle();
            return;
        }

        const attributes: Record<string, unknown> = {};
        for (const column of this._config!.columns) {
            if (column.kind !== 'attribute' || column.editable === false) {
                continue;
            }

            const kind = this._attributeKinds[column.attributeKey];
            const value = this._createDraft.attributes[column.attributeKey];
            if (isBlankAttributeValue(value, kind)) {
                continue;
            }

            attributes[column.attributeKey] = value;
        }

        this._creating = true;
        this._createError = null;
        this._render();

        try {
            const addDto: {
                attributes?: Record<string, unknown>;
                content: string;
                links?: Array<{ other_item_id: number; relationship: ItemRelationship }>;
                title: string;
                types?: string[];
            } = {
                content: '',
                title,
            };

            if (Object.keys(attributes).length > 0) {
                addDto.attributes = attributes;
            }

            if (createConfig.contextItemId != null) {
                addDto.links = [{
                    other_item_id: createConfig.contextItemId,
                    relationship: createConfig.relationship ?? 'parent',
                }];
            }

            if (this._createDraft.types.length > 0) {
                addDto.types = this._createDraft.types;
            }

            const created = await itemApi.Add(addDto);

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

    private _sortedItems(): Item[] {
        const items = [...this._items];
        if (!this._sortColumnKey || !this._sortDirection) {
            return items;
        }

        const column = this._config!.columns.find((entry) => this._keyForColumn(entry) === this._sortColumnKey);
        if (!column) {
            return items;
        }

        items.sort((left, right) => {
            const leftValue = this._sortValueForItem(left, column);
            const rightValue = this._sortValueForItem(right, column);

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

        if (this._sortDirection === 'desc') {
            items.reverse();
        }

        return items;
    }

    private _sortValueForItem(item: Item, column: ItemTableColumn): string | number | null {
        if (column.kind === 'title') {
            return item.Title.toLowerCase();
        }

        if (column.kind === 'types') {
            const typeNames = item.Types.map((typeRef) => typeRef.Name).join('\u0000').toLowerCase();
            return typeNames === '' ? null : typeNames;
        }

        return getComparableAttributeValue(item.Attributes[column.attributeKey], this._attributeKinds[column.attributeKey]);
    }

    private _toggleSort(column: ItemTableColumn): void {
        const key = this._keyForColumn(column);
        if (this._sortColumnKey !== key) {
            this._sortColumnKey = key;
            this._sortDirection = 'asc';
            this._render();
            return;
        }

        if (this._sortDirection === 'asc') {
            this._sortDirection = 'desc';
        } else if (this._sortDirection === 'desc') {
            this._sortDirection = null;
            this._sortColumnKey = null;
        } else {
            this._sortDirection = 'asc';
        }

        this._render();
    }

    private _columnCount(): number {
        return this._config!.columns.length + (this._config!.createRow?.enabled ? 1 : 0);
    }

    private _keyForColumn(column: ItemTableColumn): string {
        if (column.kind === 'attribute') {
            return `attr:${column.attributeKey}`;
        }

        return column.kind;
    }

    private _labelForColumn(column: ItemTableColumn): string {
        if (column.label) {
            return column.label;
        }

        if (column.kind === 'title') {
            return 'Title';
        }

        if (column.kind === 'types') {
            return 'Type';
        }

        return column.attributeKey;
    }

    private _readOnlyCell(value: string): HTMLElement {
        const span = document.createElement('span');
        span.textContent = value;
        return span;
    }

    private _formatAttributeValue(value: unknown): string {
        if (Array.isArray(value)) {
            return value.map((entry) => String(entry)).join(', ');
        }

        if (typeof value === 'boolean') {
            return value ? 'True' : 'False';
        }

        if (value == null) {
            return '';
        }

        return String(value);
    }

    private _isInteractiveTarget(target: EventTarget | null): boolean {
        if (!(target instanceof Element)) {
            return false;
        }

        return target.closest('button, input, select, textarea, a, chips-input, item-picker-input, item-chips-input') !== null;
    }

    private _newCreateDraft(): CreateDraftState {
        return {
            attributes: {},
            title: '',
            types: [],
        };
    }
}

if (!customElements.get('item-table-view')) {
    customElements.define('item-table-view', ItemTableView);
}
