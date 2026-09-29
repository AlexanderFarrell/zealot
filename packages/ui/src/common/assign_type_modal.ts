import { Popups } from '@websoil/engine';
import { ItemAPI } from '@zealot/api/src/item';
import { ItemTypeAPI } from '@zealot/api/src/item_type';
import { AttributeAPI } from '@zealot/api/src/attribute';
import type { Item } from '@zealot/domain/src/item';
import type { ItemType } from '@zealot/domain/src/item_type';
import {
    createAttributeValueInput,
    isBlankAttributeValue,
    loadAttributeKinds,
    type AttributeValueInputBinding,
} from '../views/attribute_value_input';

const itemApi = new ItemAPI('/api');
const itemTypeApi = new ItemTypeAPI('/api');
const attrApi = new AttributeAPI('/api');

export class AssignTypeModal extends HTMLElement {
    private item: Item | null = null;
    private onDone: (() => void) | null = null;
    private listEl: HTMLElement | null = null;
    private rendered = false;

    static show(item: Item, onDone: () => void): AssignTypeModal {
        const existing = document.querySelector('assign-type-modal') as AssignTypeModal | null;
        if (existing) {
            existing.remove();
        }
        const modal = new AssignTypeModal();
        modal.item = item;
        modal.onDone = onDone;
        (document.body ?? document.documentElement).appendChild(modal);
        return modal;
    }

    connectedCallback(): void {
        if (!this.rendered) {
            this.render();
            void this.loadTypes();
        }
    }

    private render(): void {
        this.rendered = true;
        this.classList.add('modal_background');
        this.classList.add('assign-type-modal');
        this.innerHTML = `
        <div class="inner_window assign-type-modal-window" role="dialog" aria-modal="true" aria-label="Manage types">
            <div class="tool-panel">
                <div class="tool-panel-header">
                    <h2>Manage Types</h2>
                    <button type="button" class="assign-type-modal-close" aria-label="Close">&times;</button>
                </div>
                <div class="assign-type-list">Loading…</div>
            </div>
        </div>
        `;

        this.listEl = this.querySelector('.assign-type-list');

        this.addEventListener('click', (event) => {
            if (event.target === this) this.close();
        });
        this.addEventListener('keydown', (event) => {
            if (event.key === 'Escape') {
                event.preventDefault();
                this.close();
            }
        });
        this.querySelector('.assign-type-modal-close')?.addEventListener('click', () => this.close());
    }

    private async loadTypes(): Promise<void> {
        if (!this.listEl || !this.item) return;

        try {
            const all = await itemTypeApi.get_all();
            const kinds = await loadAttributeKinds();
            const assignedNames = new Set(this.item.Types.map((t) => t.Name));

            this.listEl.innerHTML = '';

            if (all.length === 0) {
                const msg = document.createElement('p');
                msg.className = 'tool-muted';
                msg.textContent = 'No types defined.';
                this.listEl.appendChild(msg);
                return;
            }

            const sorted = [...all].sort((a, b) => a.Name.localeCompare(b.Name));

            for (const type of sorted) {
                const row = document.createElement('div');
                row.className = 'assign-type-row';
                row.dataset.typeName = type.Name;

                const nameEl = document.createElement('span');
                nameEl.className = 'assign-type-name';
                nameEl.textContent = type.Name;

                const btn = document.createElement('button');
                btn.type = 'button';

                if (assignedNames.has(type.Name)) {
                    btn.textContent = 'Remove';
                    btn.className = 'assign-type-btn assign-type-btn--remove';
                    btn.addEventListener('click', () => void this.removeType(type.Name, row, btn));
                } else {
                    btn.textContent = 'Add';
                    btn.className = 'assign-type-btn assign-type-btn--add';
                    btn.addEventListener('click', () => void this.startAdd(type, row, btn, kinds));
                }

                row.appendChild(nameEl);
                row.appendChild(btn);
                this.listEl.appendChild(row);
            }
        } catch (e) {
            if (this.listEl) {
                this.listEl.innerHTML = '';
                const err = document.createElement('p');
                err.className = 'tool-error';
                err.textContent = 'Failed to load types.';
                this.listEl.appendChild(err);
            }
        }
    }

    private async removeType(typeName: string, row: HTMLElement, btn: HTMLButtonElement): Promise<void> {
        if (!this.item) return;
        btn.disabled = true;
        btn.textContent = 'Removing…';
        try {
            await itemApi.UnassignType(this.item.ItemID, typeName);
            this.item.Types = this.item.Types.filter((t) => t.Name !== typeName);
            btn.textContent = 'Add';
            btn.className = 'assign-type-btn assign-type-btn--add';
            btn.disabled = false;
            // Re-wire button as an Add button
            const newBtn = btn.cloneNode(true) as HTMLButtonElement;
            btn.replaceWith(newBtn);
            // Need a fresh reference to the type — reload from API to get RequiredAttributes
            const freshType = await itemTypeApi.get_by_name(typeName);
            if (freshType) {
                const kinds = await loadAttributeKinds();
                newBtn.addEventListener('click', () => void this.startAdd(freshType, row, newBtn, kinds));
            }
            this.onDone?.();
        } catch (e) {
            Popups.add_error((e as Error).message ?? `Failed to remove type "${typeName}".`);
            btn.disabled = false;
            btn.textContent = 'Remove';
        }
    }

    private async startAdd(
        type: ItemType,
        row: HTMLElement,
        btn: HTMLButtonElement,
        kinds: Record<string, import('@zealot/domain/src/attribute').AttributeKind>,
    ): Promise<void> {
        if (!this.item) return;

        const missing = type.RequiredAttributes.filter((k) => !(k in this.item!.Attributes));

        if (missing.length === 0) {
            await this.assignType(type.Name, [], [], btn);
            return;
        }

        // Remove any existing inline form in this row
        row.querySelector('.assign-type-fill-form')?.remove();

        const form = document.createElement('div');
        form.className = 'assign-type-fill-form';

        const label = document.createElement('p');
        label.className = 'assign-type-fill-label';
        label.textContent = 'Fill in required attributes:';
        form.appendChild(label);

        const bindings: Array<{ key: string; binding: AttributeValueInputBinding }> = [];

        for (const key of missing) {
            const fieldRow = document.createElement('div');
            fieldRow.className = 'assign-type-fill-row';

            const keyLabel = document.createElement('label');
            keyLabel.className = 'assign-type-fill-key';
            keyLabel.textContent = key;

            const kind = kinds[key];
            const binding = createAttributeValueInput({
                attributeKey: key,
                value: null,
                ...(kind ? { kind } : {}),
            });

            bindings.push({ key, binding });
            fieldRow.appendChild(keyLabel);
            fieldRow.appendChild(binding.element);
            form.appendChild(fieldRow);
        }

        const errorEl = document.createElement('p');
        errorEl.className = 'tool-error';
        errorEl.hidden = true;
        form.appendChild(errorEl);

        const confirmBtn = document.createElement('button');
        confirmBtn.type = 'button';
        confirmBtn.textContent = 'Confirm';
        confirmBtn.className = 'assign-type-btn assign-type-btn--confirm';
        confirmBtn.addEventListener('click', () => {
            // Validate all filled
            for (const { key, binding } of bindings) {
                const v = binding.getValue();
                if (isBlankAttributeValue(v, kinds[key])) {
                    errorEl.textContent = `"${key}" is required.`;
                    errorEl.hidden = false;
                    binding.focus();
                    return;
                }
            }
            errorEl.hidden = true;
            const keys = bindings.map((b) => b.key);
            const values = bindings.map((b) => b.binding.getValue());
            void this.assignType(type.Name, keys, values, btn, form, confirmBtn);
        });

        const cancelBtn = document.createElement('button');
        cancelBtn.type = 'button';
        cancelBtn.textContent = 'Cancel';
        cancelBtn.className = 'assign-type-btn';
        cancelBtn.addEventListener('click', () => form.remove());

        form.appendChild(confirmBtn);
        form.appendChild(cancelBtn);
        row.appendChild(form);

        bindings[0]?.binding.focus();
    }

    private async assignType(
        typeName: string,
        attrKeys: string[],
        attrValues: unknown[],
        btn: HTMLButtonElement,
        form?: HTMLElement,
        confirmBtn?: HTMLButtonElement,
    ): Promise<void> {
        if (!this.item) return;

        if (confirmBtn) confirmBtn.disabled = true;
        btn.disabled = true;

        try {
            for (let i = 0; i < attrKeys.length; i++) {
                const key = attrKeys[i]!;
                await attrApi.set_value(this.item.ItemID, key, attrValues[i]);
                this.item.Attributes[key] = attrValues[i];
            }
            await itemApi.AssignType(this.item.ItemID, typeName);

            // Update local item.Types so the badge list reflects the new type
            // We don't have the full ItemTypeRef here, so use a minimal shape
            this.item.Types = [
                ...this.item.Types,
                { TypeID: -1, Name: typeName, IsSystem: false } as import('@zealot/domain/src/item_type').ItemTypeRef,
            ];

            form?.remove();
            btn.textContent = 'Remove';
            btn.className = 'assign-type-btn assign-type-btn--remove';
            btn.disabled = false;

            // Re-wire as remove button
            const newBtn = btn.cloneNode(true) as HTMLButtonElement;
            const row = btn.closest('.assign-type-row') as HTMLElement;
            btn.replaceWith(newBtn);
            newBtn.addEventListener('click', () => void this.removeType(typeName, row, newBtn));

            this.onDone?.();
        } catch (e) {
            Popups.add_error((e as Error).message ?? `Failed to assign type "${typeName}".`);
            if (confirmBtn) confirmBtn.disabled = false;
            btn.disabled = false;
        }
    }

    private close(): void {
        this.remove();
    }
}

if (!customElements.get('assign-type-modal')) {
    customElements.define('assign-type-modal', AssignTypeModal);
}
