import { Popups, getNavigator } from '@websoil/engine';
import { AttributeAPI } from '@zealot/api/src/attribute';
import { ItemAPI } from '@zealot/api/src/item';
import type { Item } from '@zealot/domain/src/item';
import type { AttributeKind } from '@zealot/domain/src/attribute';
import {
    createAttributeValueInput,
    isBlankAttributeValue,
    loadAttributeKinds,
    type AttributeValueInputBinding,
} from './attribute_value_input';

const attrApi = new AttributeAPI('/api');
const itemApi = new ItemAPI('/api');
const EXPANDED_KEY = 'zealot.attributes.expanded';

function readExpanded(): boolean {
    try {
        return localStorage.getItem(EXPANDED_KEY) === '1';
    } catch {
        return false;
    }
}

function writeExpanded(expanded: boolean): void {
    try {
        localStorage.setItem(EXPANDED_KEY, expanded ? '1' : '0');
    } catch {
        // The choice just won't persist.
    }
}

function numericIds(value: unknown): number[] {
    return (Array.isArray(value) ? value : [value]).filter((v): v is number => typeof v === 'number');
}

// Phones keep a tap on a chip as a toggle; desktop lets Parent chips navigate.
const desktopQuery = '(min-width: 769px) and (not ((max-width: 1100px) and (hover: none) and (pointer: coarse)))';

/**
 * <attribute-editor>
 *
 * Renders a complete, type-aware attribute editor for an Item. Shows one row
 * per attribute with a typed value control, a rename-on-click key input, and a
 * delete button. An "add attribute" row sits at the bottom.
 *
 * Usage:
 *   const editor = new AttributeEditor();
 *   container.appendChild(editor);
 *   editor.init(item);
 */
export class AttributeEditor extends HTMLElement {
    private _item: Item | null = null;
    public onIconChange: (() => void) | null = null;
    // One summary line; clicking it reveals the table (Z240, Z253). The choice is
    // remembered per browser so it follows you from item to item.
    private _expanded = readExpanded();

    init(item: Item): this {
        this._item = item;
        if (this.isConnected) {
            void this._render();
        }
        return this;
    }

    setItem(item: Item): void {
        this._item = item;
        void this._render();
    }

    connectedCallback() {
        if (this._item) {
            void this._render();
        }
    }

    private async _render() {
        if (!this._item) return;
        const item = this._item;

        this.innerHTML = '';

        const kinds = await loadAttributeKinds();

        const datalistId = `attr-key-list-${Math.random().toString(36).slice(2)}`;
        const datalist = document.createElement('datalist');
        datalist.id = datalistId;
        for (const k of Object.keys(kinds)) {
            const opt = document.createElement('option');
            opt.value = k;
            datalist.appendChild(opt);
        }

        this.classList.toggle('attribute-editor-host--expanded', this._expanded);
        this.appendChild(this._buildSummary(item));

        const container = document.createElement('div');
        container.className = 'attribute-editor';
        container.appendChild(datalist);

        for (const [key, value] of Object.entries(item.Attributes)) {
            container.appendChild(this._buildRow(item, key, value, kinds, datalistId));
        }

        container.appendChild(this._buildAddRow(item, kinds, datalistId));
        this.appendChild(container);
    }

    private _buildSummary(item: Item): HTMLElement {
        const button = document.createElement('button');
        button.type = 'button';
        button.className = 'attribute-summary';
        button.setAttribute('aria-expanded', String(this._expanded));
        button.title = this._expanded ? 'Hide attributes' : 'Show attributes';

        const entries = Object.entries(item.Attributes);
        if (entries.length === 0) {
            const empty = document.createElement('span');
            empty.className = 'attribute-summary-empty';
            empty.textContent = 'Attributes';
            button.appendChild(empty);
        }
        const chips = entries.map(([key, value]) => {
            const text = Array.isArray(value)
                ? value.map((entry) => (typeof entry === 'object' ? JSON.stringify(entry) : String(entry))).join(', ')
                : typeof value === 'object' && value !== null ? JSON.stringify(value) : String(value);
            return { key, value, label: key === 'Icon' ? text : `${key}: ${text}`, text };
        }).filter((chip) => chip.text);
        // Parent leads; the rest go smallest pill first so short ones stay
        // visible before the scroll.
        chips.sort((a, b) => {
            if (a.key === 'Parent' || b.key === 'Parent') return a.key === 'Parent' ? -1 : 1;
            return a.label.length - b.label.length;
        });
        for (const { key, value, label } of chips) {
            const parentIds = key === 'Parent' ? numericIds(value) : [];
            if (parentIds.length > 0) {
                this._appendParentChips(button, parentIds);
                continue;
            }
            const chip = document.createElement('span');
            chip.className = 'attribute-summary-chip';
            chip.textContent = label;
            button.appendChild(chip);
        }
        const caret = document.createElement('span');
        caret.className = 'attribute-summary-caret';
        caret.setAttribute('aria-hidden', 'true');
        caret.textContent = this._expanded ? '▲' : '▼';
        button.appendChild(caret);

        button.addEventListener('click', () => {
            this._expanded = !this._expanded;
            writeExpanded(this._expanded);
            void this._render();
        });
        return button;
    }

    // One chip per parent so each can be opened (Z253). The first leads with
    // "Parent:"; each shows the parent's title, trimmed to keep the pill short.
    private _appendParentChips(button: HTMLElement, ids: number[]): void {
        ids.forEach((id, index) => {
            const prefix = index === 0 ? 'Parent: ' : '';
            const chip = document.createElement('span');
            chip.className = 'attribute-summary-chip attribute-summary-chip--link';
            chip.setAttribute('role', 'link');
            chip.textContent = `${prefix}${id}`;
            chip.addEventListener('click', (event) => {
                if (!window.matchMedia(desktopQuery).matches) return;
                event.stopPropagation();
                getNavigator().openItemById(id);
            });
            button.appendChild(chip);
            void itemApi.GetById(id).then((parent) => {
                const title = parent.Title;
                chip.textContent = `${prefix}${title.length > 25 ? `${title.slice(0, 25).trimEnd()}…` : title}`;
            }).catch(() => {
                // Keep the id form when the title cannot be loaded.
            });
        });
    }

    private async _renderAndFocusNewKey(): Promise<void> {
        await this._render();
        const rows = this.querySelectorAll('.attribute-editor .attribute');
        const lastRow = rows[rows.length - 1];
        lastRow?.querySelector<HTMLInputElement>('input[placeholder="New key"]')?.focus();
    }

    private _buildRow(
        item: Item,
        key: string,
        value: any,
        kinds: Record<string, AttributeKind>,
        datalistId: string,
    ): HTMLElement {
        const row = document.createElement('div');
        row.className = 'attribute';

        // --- Key input ---
        const keyInput = document.createElement('input');
        keyInput.type = 'text';
        keyInput.value = key;
        keyInput.title = 'Rename attribute (blur to save)';
        keyInput.setAttribute('list', datalistId);
        keyInput.addEventListener('blur', async () => {
            const newKey = keyInput.value.trim();
            if (!newKey || newKey === key) return;
            try {
                await attrApi.rename(item.ItemID, key, newKey);
                item.Attributes[newKey] = item.Attributes[key];
                delete item.Attributes[key];
                if (key === 'Icon' || newKey === 'Icon') {
                    this.onIconChange?.();
                }
                void this._render();
            } catch (e) {
                Popups.add_error((e as Error).message ?? 'Failed to rename attribute.');
                keyInput.value = key;
            }
        });

        // --- Value control wrapper ---
        const valueSpan = document.createElement('span');
        valueSpan.setAttribute('name', 'value_view');

        const kind = kinds[key];
        const controlOptions: Parameters<typeof createAttributeValueInput>[0] = {
            attributeKey: key,
            value,
            onCommit: async (nextValue) => {
                const result = await this._saveValue(item, key, nextValue, kind);
                if (result === 'saved') {
                    item.Attributes[key] = nextValue;
                    if (key === 'Icon') {
                        this.onIconChange?.();
                    }
                    return;
                }

                if (result === 'removed') {
                    delete item.Attributes[key];
                    if (key === 'Icon') {
                        this.onIconChange?.();
                    }
                }

                void this._render();
            },
        };
        if (kind) {
            controlOptions.kind = kind;
        }

        const control = createAttributeValueInput(controlOptions);
        valueSpan.appendChild(control.element);

        // --- Delete button ---
        const delBtn = document.createElement('button');
        delBtn.type = 'button';
        delBtn.textContent = '×';
        delBtn.title = 'Delete attribute';
        delBtn.addEventListener('click', async () => {
            try {
                await attrApi.remove(item.ItemID, key);
                delete item.Attributes[key];
                if (key === 'Icon') {
                    this.onIconChange?.();
                }
                void this._render();
            } catch (e) {
                Popups.add_error((e as Error).message ?? 'Failed to delete attribute.');
            }
        });

        row.appendChild(keyInput);
        row.appendChild(valueSpan);
        row.appendChild(delBtn);
        return row;
    }

    private _buildAddRow(item: Item, kinds: Record<string, AttributeKind>, datalistId: string): HTMLElement {
        const row = document.createElement('div');
        row.className = 'attribute';

        const keyInput = document.createElement('input');
        keyInput.type = 'text';
        keyInput.placeholder = 'New key';
        keyInput.setAttribute('list', datalistId);

        const valueSpan = document.createElement('span');
        valueSpan.setAttribute('name', 'value_view');

        // Start with a plain text input; swapped when key matches a known kind.
        let currentBinding: AttributeValueInputBinding = createAttributeValueInput({
            attributeKey: '',
            value: null,
        });
        valueSpan.appendChild(currentBinding.element);

        const resetBinding = () => {
            currentBinding = createAttributeValueInput({ attributeKey: '', value: null });
            valueSpan.innerHTML = '';
            valueSpan.appendChild(currentBinding.element);
        };

        keyInput.addEventListener('input', () => {
            const k = keyInput.value.trim();
            const kind = kinds[k];
            currentBinding = createAttributeValueInput({ attributeKey: k, value: null, ...(kind ? { kind } : {}) });
            valueSpan.innerHTML = '';
            valueSpan.appendChild(currentBinding.element);
        });

        const addBtn = document.createElement('button');
        addBtn.type = 'button';
        addBtn.textContent = '+';

        const doAdd = async () => {
            const k = keyInput.value.trim();
            if (!k || k in item.Attributes) return;
            const v = currentBinding.getValue();
            if (isBlankAttributeValue(v, kinds[k])) return;
            try {
                await attrApi.set_value(item.ItemID, k, v);
                item.Attributes[k] = v;
                if (k === 'Icon') {
                    this.onIconChange?.();
                }
                keyInput.value = '';
                resetBinding();
                void this._renderAndFocusNewKey();
            } catch (e) {
                Popups.add_error((e as Error).message ?? 'Failed to add attribute.');
            }
        };

        addBtn.addEventListener('click', doAdd);
        keyInput.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                e.preventDefault();
                void doAdd();
            }
        });

        row.appendChild(keyInput);
        row.appendChild(valueSpan);
        row.appendChild(addBtn);
        return row;
    }

    private async _saveValue(
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
        } catch (e) {
            Popups.add_error((e as Error).message ?? `Failed to save attribute "${key}".`);
            return 'failed';
        }
    }
}

if (!customElements.get('attribute-editor')) {
    customElements.define('attribute-editor', AttributeEditor);
}
