import { Events, ItemEvents, getNavigator } from '@websoil/engine';
import { ItemAPI } from '@zealot/api/src/item';
import { ItemTypeAPI } from '@zealot/api/src/item_type';
import type { AttributeKind } from '@zealot/domain/src/attribute';
import type { ItemType } from '@zealot/domain/src/item_type';
import { ItemSearchInline } from '../views/item_search_inline';
import {
    createAttributeValueInput,
    isBlankAttributeValue,
    loadAttributeKinds,
    type AttributeValueInputBinding,
} from '../views/attribute_value_input';

const itemApi = new ItemAPI('/api');
const itemTypeApi = new ItemTypeAPI('/api');

export class AddItemModal extends HTMLElement {
    private titleInput: HTMLInputElement | null = null;
    private parentSearch: ItemSearchInline | null = null;
    private typeSelect: HTMLSelectElement | null = null;
    private errorEl: HTMLDivElement | null = null;
    private submitButton: HTMLButtonElement | null = null;
    private attrContainer: HTMLElement | null = null;
    private rendered = false;
    private creating = false;
    private selectedType: ItemType | null = null;
    private attributeKinds: Record<string, AttributeKind> = {};
    private attributeBindings: Array<{ key: string; binding: AttributeValueInputBinding }> = [];

    static show(): AddItemModal {
        const existing = document.querySelector('add-item-modal') as AddItemModal | null;
        if (existing) {
            existing.focusTitle();
            return existing;
        }

        const modal = new AddItemModal();
        (document.body ?? document.documentElement).appendChild(modal);
        queueMicrotask(() => modal.focusTitle());
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
        this.classList.add('add-item-modal');
        this.innerHTML = `
        <form class="inner_window add-item-modal-window" role="dialog" aria-modal="true" aria-label="Add item">
            <div class="tool-panel">
                <div class="tool-panel-header">
                    <h2>Add Item</h2>
                </div>
                <label class="tool-field">
                    <span class="tool-label">Title</span>
                    <input name="title" type="text" required>
                </label>
                <label class="tool-field">
                    <span class="tool-label">Parent Item</span>
                    <div data-role="parent-search"></div>
                </label>
                <label class="tool-field">
                    <span class="tool-label">Item Type</span>
                    <select name="item_type" disabled>
                        <option value="">Loading types…</option>
                    </select>
                </label>
                <div data-role="attr-fields"></div>
                <div class="tool-error" data-role="error" hidden></div>
                <div class="add-item-modal-actions">
                    <button type="button" data-role="cancel">Cancel</button>
                    <button type="submit" data-role="submit">Create</button>
                </div>
            </div>
        </form>
        `;

        this.titleInput = this.querySelector<HTMLInputElement>('[name="title"]');
        this.typeSelect = this.querySelector<HTMLSelectElement>('[name="item_type"]');
        this.errorEl = this.querySelector<HTMLDivElement>('[data-role="error"]');
        this.submitButton = this.querySelector<HTMLButtonElement>('[data-role="submit"]');
        this.attrContainer = this.querySelector<HTMLElement>('[data-role="attr-fields"]');

        const parentHost = this.querySelector<HTMLElement>('[data-role="parent-search"]');
        const parentSearch = new ItemSearchInline();
        parentSearch.placeholder = 'Search parent item…';
        parentHost?.appendChild(parentSearch);
        this.parentSearch = parentSearch;

        this.addEventListener('click', (event) => {
            if (event.target === this) {
                this.close();
            }
        });

        this.addEventListener('keydown', (event) => {
            if (event.key === 'Escape') {
                event.preventDefault();
                this.close();
            }
        });

        this.querySelector('[data-role="cancel"]')?.addEventListener('click', () => {
            this.close();
        });

        this.querySelector('form')?.addEventListener('submit', (event) => {
            event.preventDefault();
            void this.submit();
        });
    }

    private async loadTypes(): Promise<void> {
        if (!this.typeSelect) {
            return;
        }

        void loadAttributeKinds().then((kinds) => { this.attributeKinds = kinds; });

        try {
            const types = await itemTypeApi.get_all();
            const selectedValue = this.typeSelect.value;
            const sortedTypes = [...types].sort((left, right) => left.Name.localeCompare(right.Name));
            this.typeSelect.innerHTML = '<option value="">None</option>';
            sortedTypes.forEach((type) => {
                const option = document.createElement('option');
                option.value = type.Name;
                option.textContent = type.Name;
                this.typeSelect?.appendChild(option);
            });
            this.typeSelect.value = selectedValue;
        } catch {
            this.typeSelect.innerHTML = '<option value="">None</option>';
        } finally {
            this.typeSelect.disabled = false;
        }

        this.typeSelect.addEventListener('change', () => {
            const name = this.typeSelect?.value ?? '';
            if (!name) {
                this.selectedType = null;
                this.attributeBindings = [];
                this.renderAttributeFields();
                return;
            }
            void itemTypeApi.get_by_name(name).then((type) => {
                this.selectedType = type;
                this.renderAttributeFields();
            });
        });
    }

    private renderAttributeFields(): void {
        if (!this.attrContainer) return;
        this.attrContainer.innerHTML = '';
        this.attributeBindings = [];

        const type = this.selectedType;
        if (!type?.RequiredAttributes.length) return;

        for (const key of type.RequiredAttributes) {
            const kind = this.attributeKinds[key];
            const field = document.createElement('label');
            field.className = 'tool-field';

            const lbl = document.createElement('span');
            lbl.className = 'tool-label';
            lbl.textContent = key;

            const opts: Parameters<typeof createAttributeValueInput>[0] = {
                allowEmpty: true,
                attributeKey: key,
                value: undefined,
                onValueChange: () => {},
            };
            if (kind) opts.kind = kind;
            const binding = createAttributeValueInput(opts);

            this.attributeBindings.push({ key, binding });
            field.appendChild(lbl);
            field.appendChild(binding.element);
            this.attrContainer.appendChild(field);
        }
    }

    private async submit(): Promise<void> {
        if (this.creating) {
            return;
        }

        const title = this.titleInput?.value.trim() ?? '';
        if (!title) {
            this.setError('Title is required.');
            this.focusTitle();
            return;
        }

        for (const { key, binding } of this.attributeBindings) {
            if (isBlankAttributeValue(binding.getValue(), this.attributeKinds[key])) {
                this.setError(`"${key}" is required.`);
                binding.focus();
                return;
            }
        }

        this.creating = true;
        this.setError('');
        this.updateSubmitState();

        try {
            const typeName = this.typeSelect?.value.trim() ?? '';
            const parentId = this.parentSearch?.value?.ItemID;
            const dto: {
                content: '',
                title: string;
                links?: Array<{ other_item_id: number; relationship: 'parent' }>;
                types?: string[];
                attributes?: Record<string, unknown>;
            } = {
                content: '',
                title,
            };

            if (parentId != null) {
                dto.links = [{ other_item_id: parentId, relationship: 'parent' }];
            }

            if (typeName) {
                dto.types = [typeName];
            }

            if (this.attributeBindings.length > 0) {
                const attrs: Record<string, unknown> = {};
                for (const { key, binding } of this.attributeBindings) {
                    attrs[key] = binding.getValue();
                }
                dto.attributes = attrs;
            }

            const created = await itemApi.Add(dto);

            Events.emit(ItemEvents.created, created);
            this.close();
            getNavigator().openItem(created.Title);
        } catch (error) {
            this.setError(await this.getErrorMessage(error));
        } finally {
            this.creating = false;
            this.updateSubmitState();
        }
    }

    private async getErrorMessage(error: unknown): Promise<string> {
        const response = (error as Error & { response?: Response }).response;
        if (response) {
            try {
                const text = await response.text();
                if (text.trim() !== '') {
                    return text;
                }
            } catch {
                // Ignore response parsing errors and fall back to the generic message.
            }
        }

        return (error as Error).message ?? 'Failed to create item.';
    }

    private updateSubmitState(): void {
        if (!this.submitButton) {
            return;
        }

        this.submitButton.disabled = this.creating;
        this.submitButton.textContent = this.creating ? 'Creating…' : 'Create';
    }

    private setError(message: string): void {
        if (!this.errorEl) {
            return;
        }

        this.errorEl.textContent = message;
        this.errorEl.hidden = message.trim() === '';
    }

    private focusTitle(): void {
        window.requestAnimationFrame(() => {
            this.titleInput?.focus();
            this.titleInput?.select();
        });
    }

    private close(): void {
        this.remove();
    }
}

if (!customElements.get('add-item-modal')) {
    customElements.define('add-item-modal', AddItemModal);
}
