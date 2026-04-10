import { BaseElementEmpty, getNavigator } from '@websoil/engine';
import { AttributeKindAPI } from '@zealot/api/src/attribute_kind';
import { ItemAPI } from '@zealot/api/src/item';
import { ItemTypeAPI } from '@zealot/api/src/item_type';
import type { AttributeKind } from '@zealot/domain/src/attribute';
import type { Item } from '@zealot/domain/src/item';
import type { ItemType } from '@zealot/domain/src/item_type';
import { LoadingSpinner } from '../common/loading_spinner';
import { ItemTableView, type ItemTableColumn } from '../views/item_table_view';

const itemTypeApi = new ItemTypeAPI('/api');
const attrKindApi = new AttributeKindAPI('/api');
const itemApi = new ItemAPI('/api');

export class TypeScreen extends BaseElementEmpty {
    private typeTitle: string | null = null;
    private renderId = 0;

    async render() {
        const renderId = ++this.renderId;
        const title = this.typeTitle;

        this.className = 'type-screen';
        this.innerHTML = '';

        if (!title) {
            const message = document.createElement('p');
            message.className = 'tool-error';
            message.textContent = 'No type selected.';
            this.appendChild(message);
            return;
        }

        const loading = new LoadingSpinner();
        this.appendChild(loading);

        const [typeResult, kindsResult, itemsResult] = await Promise.allSettled([
            itemTypeApi.get_by_name(title),
            attrKindApi.get_all(),
            itemApi.GetAll(title),
        ]);

        if (renderId !== this.renderId) {
            return;
        }

        this.innerHTML = '';

        if (typeResult.status === 'rejected') {
            this.renderError(typeResult.reason, 'Failed to load the item type.');
            return;
        }

        if (typeResult.value == null) {
            this.renderNotFound(title);
            return;
        }

        const attributeKinds = kindsResult.status === 'fulfilled' ? kindsResult.value : [];
        const items = itemsResult.status === 'fulfilled' ? itemsResult.value : null;
        const kindsError = kindsResult.status === 'rejected'
            ? this.messageForError(kindsResult.reason, 'Failed to load attribute kinds.')
            : null;
        const itemsError = itemsResult.status === 'rejected'
            ? this.messageForError(itemsResult.reason, 'Failed to load items.')
            : null;

        this.renderType(typeResult.value, attributeKinds, kindsError, items, itemsError);
    }

    init(title: string): this {
        this.typeTitle = title;
        if (this.isConnected) {
            void this.render();
        }
        return this;
    }

    private renderType(
        itemType: ItemType,
        attributeKinds: AttributeKind[],
        kindsError: string | null,
        items: Item[] | null,
        itemsError: string | null,
    ): void {
        const shell = document.createElement('div');
        shell.className = 'type-screen-shell';

        shell.appendChild(this.buildHeader(itemType));
        shell.appendChild(this.buildAssignedKindsSection(itemType, attributeKinds, kindsError));
        shell.appendChild(this.buildItemsSection(itemType, items, itemsError));

        this.appendChild(shell);
    }

    private buildHeader(itemType: ItemType): HTMLElement {
        const header = document.createElement('div');
        header.className = 'type-screen-header';

        const backButton = document.createElement('button');
        backButton.type = 'button';
        backButton.textContent = 'Back to Types';
        backButton.addEventListener('click', () => {
            getNavigator().openTypes();
        });

        const titleRow = document.createElement('div');
        titleRow.className = 'type-screen-title-row';

        if (itemType.IsSystem) {
            const title = document.createElement('h1');
            title.textContent = itemType.Name;
            titleRow.appendChild(title);
        } else {
            const input = document.createElement('input');
            input.type = 'text';
            input.className = 'type-screen-title-input';
            input.value = itemType.Name;
            input.addEventListener('keydown', (event) => {
                if (event.key === 'Enter') {
                    event.preventDefault();
                    input.blur();
                } else if (event.key === 'Escape') {
                    input.value = itemType.Name;
                    input.blur();
                }
            });
            input.addEventListener('blur', () => {
                void this.saveTitle(itemType, input);
            });
            titleRow.appendChild(input);
        }

        if (itemType.IsSystem) {
            const badge = document.createElement('span');
            badge.className = 'tool-badge';
            badge.textContent = 'System';
            titleRow.appendChild(badge);
        }

        header.append(backButton, titleRow);
        return header;
    }

    private buildAssignedKindsSection(
        itemType: ItemType,
        attributeKinds: AttributeKind[],
        kindsError: string | null,
    ): HTMLElement {
        const section = document.createElement('section');
        section.className = 'type-screen-section';

        const header = document.createElement('div');
        header.className = 'type-screen-section-header';

        const title = document.createElement('h2');
        title.textContent = 'Assigned Attribute Kinds';
        header.appendChild(title);

        section.appendChild(header);

        if (kindsError) {
            const error = document.createElement('p');
            error.className = 'tool-error';
            error.textContent = kindsError;
            section.appendChild(error);
        }

        const kindsByKey = new Map(attributeKinds.map((kind) => [kind.Key, kind]));
        const assignedKeys = [...itemType.RequiredAttributes].sort((left, right) => left.localeCompare(right));

        if (assignedKeys.length === 0) {
            const empty = document.createElement('p');
            empty.className = 'tool-muted';
            empty.textContent = 'No attribute kinds assigned.';
            section.appendChild(empty);
        } else {
            const table = document.createElement('table');
            table.className = 'type-attribute-table';
            table.innerHTML = `
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>Type</th>
                        <th></th>
                    </tr>
                </thead>
            `;

            const tbody = document.createElement('tbody');
            assignedKeys.forEach((key) => {
                const row = document.createElement('tr');
                const kind = kindsByKey.get(key);

                const name = document.createElement('td');
                name.textContent = key;

                const type = document.createElement('td');
                type.textContent = kind?.BaseType ?? 'unknown';

                const actions = document.createElement('td');
                const removeButton = document.createElement('button');
                removeButton.type = 'button';
                removeButton.textContent = 'Remove';
                removeButton.disabled = itemType.IsSystem;
                removeButton.addEventListener('click', () => {
                    void this.removeAttributeKind(itemType, key);
                });
                actions.appendChild(removeButton);

                row.append(name, type, actions);
                tbody.appendChild(row);
            });

            table.appendChild(tbody);
            section.appendChild(table);
        }

        const availableKinds = attributeKinds
            .filter((kind) => !itemType.RequiredAttributes.includes(kind.Key))
            .sort((left, right) => left.Key.localeCompare(right.Key));

        const controls = document.createElement('div');
        controls.className = 'type-screen-add-attribute';

        const select = document.createElement('select');
        if (availableKinds.length === 0) {
            const option = document.createElement('option');
            option.value = '';
            option.textContent = 'No attribute kinds available';
            select.appendChild(option);
        } else {
            availableKinds.forEach((kind) => {
                const option = document.createElement('option');
                option.value = kind.Key;
                option.textContent = `${kind.Key} (${kind.BaseType})`;
                select.appendChild(option);
            });
        }
        select.disabled = itemType.IsSystem || availableKinds.length === 0;

        const addButton = document.createElement('button');
        addButton.type = 'button';
        addButton.textContent = 'Add Attribute Kind';
        addButton.disabled = itemType.IsSystem || availableKinds.length === 0;
        addButton.addEventListener('click', () => {
            if (select.value) {
                void this.addAttributeKind(itemType, select.value);
            }
        });

        controls.append(select, addButton);
        section.appendChild(controls);

        return section;
    }

    private buildItemsSection(itemType: ItemType, items: Item[] | null, itemsError: string | null): HTMLElement {
        const section = document.createElement('section');
        section.className = 'type-screen-section';

        const header = document.createElement('div');
        header.className = 'type-screen-section-header';
        const title = document.createElement('h2');
        title.textContent = 'Items of This Type';
        header.appendChild(title);
        section.appendChild(header);

        if (itemsError) {
            const error = document.createElement('p');
            error.className = 'tool-error';
            error.textContent = itemsError;
            section.appendChild(error);
            return section;
        }

        const table = new ItemTableView();
        const columns: ItemTableColumn[] = [
            { kind: 'title', editable: false, label: 'Title' },
            ...itemType.RequiredAttributes.map((attributeKey) => ({
                attributeKey,
                kind: 'attribute' as const,
                editable: false,
                label: attributeKey,
            })),
        ];

        table.init({
            columns,
            emptyMessage: `No items assigned to ${itemType.Name}.`,
            items: items ?? [],
            onOpenItem: (item) => {
                getNavigator().openItem(item.Title);
            },
        });
        section.appendChild(table);

        return section;
    }

    private renderNotFound(title: string): void {
        const message = document.createElement('div');
        message.className = 'type-screen-not-found';

        const heading = document.createElement('h1');
        heading.textContent = 'Type not found';
        const copy = document.createElement('p');
        copy.className = 'tool-muted';
        copy.textContent = `No item type named "${title}" exists.`;

        const backButton = document.createElement('button');
        backButton.type = 'button';
        backButton.textContent = 'Back to Types';
        backButton.addEventListener('click', () => {
            getNavigator().openTypes();
        });

        message.append(heading, copy, backButton);
        this.appendChild(message);
    }

    private renderError(error: unknown, fallback: string): void {
        const message = document.createElement('p');
        message.className = 'tool-error';
        message.textContent = this.messageForError(error, fallback);
        this.appendChild(message);
    }

    private async saveTitle(itemType: ItemType, input: HTMLInputElement): Promise<void> {
        const nextName = input.value.trim();
        const previousName = itemType.Name;

        if (!nextName || nextName === previousName) {
            input.value = previousName;
            return;
        }

        try {
            const updated = await itemTypeApi.update(itemType.TypeID, {
                type_id: itemType.TypeID,
                name: nextName,
            });
            this.typeTitle = updated.Name;
            getNavigator().openType(updated.Name, 'replace');
        } catch (error) {
            input.value = previousName;
            this.renderInlineError(input, await this.getErrorMessage(error, 'Failed to rename item type.'));
        }
    }

    private async addAttributeKind(itemType: ItemType, key: string): Promise<void> {
        try {
            await itemTypeApi.assign(itemType.TypeID, [key]);
            void this.render();
        } catch (error) {
            this.showSectionError(await this.getErrorMessage(error, `Failed to assign "${key}".`));
        }
    }

    private async removeAttributeKind(itemType: ItemType, key: string): Promise<void> {
        try {
            await itemTypeApi.unassign(itemType.TypeID, [key]);
            void this.render();
        } catch (error) {
            this.showSectionError(await this.getErrorMessage(error, `Failed to remove "${key}".`));
        }
    }

    private renderInlineError(anchor: HTMLElement, message: string): void {
        const existing = this.querySelector('[data-type-inline-error="true"]');
        existing?.remove();

        const error = document.createElement('p');
        error.className = 'tool-error';
        error.dataset.typeInlineError = 'true';
        error.textContent = message;
        anchor.parentElement?.appendChild(error);
    }

    private showSectionError(message: string): void {
        const existing = this.querySelector('[data-type-action-error="true"]');
        existing?.remove();

        const error = document.createElement('p');
        error.className = 'tool-error';
        error.dataset.typeActionError = 'true';
        error.textContent = message;
        this.prepend(error);
    }

    private messageForError(error: unknown, fallback: string): string {
        return error instanceof Error && error.message ? error.message : fallback;
    }

    private async getErrorMessage(error: unknown, fallback: string): Promise<string> {
        const response = (error as Error & { response?: Response }).response;
        if (response) {
            try {
                const text = await response.text();
                if (text.trim() !== '') {
                    return text;
                }
            } catch {
                // Ignore response parsing errors and fall back.
            }
        }

        return this.messageForError(error, fallback);
    }
}

customElements.define('type-screen', TypeScreen);
