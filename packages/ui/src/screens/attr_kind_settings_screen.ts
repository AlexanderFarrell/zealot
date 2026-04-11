import { BaseElementEmpty } from '@websoil/engine';
import { AttributeKindAPI } from '@zealot/api/src/attribute_kind';
import type { AttributeKind } from '@zealot/domain/src/attribute';
import { AttributeBaseTypesArray } from '@zealot/domain/src/attribute';
import type { AddAttributeKindDto, AttributeBaseType, AttributeConfig } from '@zealot/domain/src/attribute';
import { ConfirmDialog } from '../common/confirm_dialog';
import { LoadingSpinner } from '../common/loading_spinner';

const attributeKindApi = new AttributeKindAPI('/api');

export class AttrKindSettingsScreen extends BaseElementEmpty {
    private createError: string | null = null;
    private creating = false;
    private renderId = 0;

    async render() {
        const renderId = ++this.renderId;

        this.className = 'attr-kind-settings-screen';
        this.innerHTML = '';

        const shell = document.createElement('div');
        shell.className = 'attr-kind-settings-shell';

        const header = document.createElement('div');
        header.className = 'attr-kind-settings-header';

        const heading = document.createElement('h2');
        heading.textContent = 'Attribute Kinds';
        const copy = document.createElement('p');
        copy.className = 'tool-muted';
        copy.textContent = 'Manage the attribute kinds used to describe items.';
        header.append(heading, copy);

        const form = this.buildAddForm();
        shell.append(header, form);

        if (this.createError) {
            const error = document.createElement('p');
            error.className = 'tool-error';
            error.textContent = this.createError;
            shell.appendChild(error);
        }

        const content = document.createElement('div');
        content.className = 'attr-kind-settings-content';
        content.appendChild(new LoadingSpinner());
        shell.appendChild(content);

        this.appendChild(shell);

        try {
            const kinds = await attributeKindApi.get_all();
            if (renderId !== this.renderId) {
                return;
            }
            this.renderKinds(content, kinds);
        } catch (error) {
            if (renderId !== this.renderId) {
                return;
            }
            content.innerHTML = '';
            const message = document.createElement('p');
            message.className = 'tool-error';
            message.textContent = error instanceof Error && error.message
                ? error.message
                : 'Failed to load attribute kinds.';
            content.appendChild(message);
        }
    }

    private buildAddForm(): HTMLFormElement {
        const form = document.createElement('form');
        form.className = 'attr-kind-settings-create';

        const keyField = document.createElement('label');
        keyField.className = 'tool-field';
        keyField.innerHTML = '<span class="tool-label">Key</span>';
        const keyInput = document.createElement('input');
        keyInput.type = 'text';
        keyInput.name = 'key';
        keyInput.placeholder = 'e.g. priority';
        keyInput.disabled = this.creating;
        keyField.appendChild(keyInput);

        const descField = document.createElement('label');
        descField.className = 'tool-field';
        descField.innerHTML = '<span class="tool-label">Description</span>';
        const descInput = document.createElement('input');
        descInput.type = 'text';
        descInput.name = 'description';
        descInput.placeholder = 'Optional description';
        descInput.disabled = this.creating;
        descField.appendChild(descInput);

        const typeField = document.createElement('label');
        typeField.className = 'tool-field';
        typeField.innerHTML = '<span class="tool-label">Base Type</span>';
        const typeSelect = buildBaseTypeSelect();
        typeSelect.disabled = this.creating;
        typeField.appendChild(typeSelect);

        const submit = document.createElement('button');
        submit.type = 'submit';
        submit.disabled = this.creating;
        submit.textContent = this.creating ? 'Adding…' : 'Add Kind';

        form.append(keyField, descField, typeField, submit);

        form.addEventListener('submit', (e) => {
            e.preventDefault();
            const key = keyInput.value.trim();
            const description = descInput.value.trim();
            const base_type = typeSelect.value as AttributeBaseType;
            if (!key) {
                this.createError = 'Key is required.';
                void this.render();
                return;
            }
            void this.addKind({ key, description, base_type, config: defaultConfigForType(base_type) });
        });

        return form;
    }

    private renderKinds(container: HTMLElement, kinds: AttributeKind[]): void {
        container.innerHTML = '';

        const systemKinds = kinds.filter((k) => k.IsSystem);
        const userKinds = kinds.filter((k) => !k.IsSystem);

        const systemSection = document.createElement('div');
        systemSection.className = 'attr-kind-section';
        const systemHeading = document.createElement('h3');
        systemHeading.textContent = 'System Kinds';
        const systemNote = document.createElement('p');
        systemNote.className = 'tool-muted';
        systemNote.textContent = 'These are built in and cannot be edited.';
        systemSection.append(systemHeading, systemNote);

        if (systemKinds.length > 0) {
            systemSection.appendChild(buildKindTable(systemKinds, true, () => {}));
        }

        const userSection = document.createElement('div');
        userSection.className = 'attr-kind-section';
        const userHeading = document.createElement('h3');
        userHeading.textContent = 'User Kinds';
        userSection.appendChild(userHeading);

        if (userKinds.length === 0) {
            const empty = document.createElement('p');
            empty.className = 'tool-muted';
            empty.textContent = 'Attribute kinds you add will appear here.';
            userSection.appendChild(empty);
        } else {
            userSection.appendChild(buildKindTable(userKinds, false, (kind) => {
                void this.deleteKind(kind);
            }));
        }

        container.append(systemSection, userSection);
    }

    private async addKind(dto: AddAttributeKindDto): Promise<void> {
        if (this.creating) {
            return;
        }
        this.creating = true;
        this.createError = null;
        void this.render();

        try {
            await attributeKindApi.add(dto);
        } catch (error) {
            this.createError = await getErrorMessage(error, 'Failed to add attribute kind.');
        }

        this.creating = false;
        void this.render();
    }

    private async deleteKind(kind: AttributeKind): Promise<void> {
        const confirmed = await ConfirmDialog.show(`Delete the "${kind.Key}" attribute kind?`);
        if (!confirmed) {
            return;
        }

        try {
            await attributeKindApi.remove(kind.KindID);
            void this.render();
        } catch (error) {
            this.createError = await getErrorMessage(error, 'Failed to delete attribute kind.');
            void this.render();
        }
    }
}

function buildKindTable(
    kinds: AttributeKind[],
    readonly: boolean,
    onDelete: (kind: AttributeKind) => void,
): HTMLTableElement {
    const table = document.createElement('table');
    table.className = 'attr-kind-table';
    table.innerHTML = `
        <thead>
            <tr>
                <th>Key</th>
                <th>Description</th>
                <th>Base Type</th>
                ${readonly ? '' : '<th></th>'}
            </tr>
        </thead>
    `;

    const tbody = document.createElement('tbody');
    kinds.forEach((kind) => {
        const row = document.createElement('tr');

        if (readonly) {
            const keyCell = document.createElement('td');
            keyCell.textContent = kind.Key;
            const descCell = document.createElement('td');
            descCell.textContent = kind.Description;
            const typeCell = document.createElement('td');
            typeCell.textContent = kind.BaseType;
            row.append(keyCell, descCell, typeCell);
        } else {
            const keyCell = document.createElement('td');
            const keyInput = document.createElement('input');
            keyInput.type = 'text';
            keyInput.value = kind.Key;
            keyInput.addEventListener('change', () => {
                kind.Key = keyInput.value;
                void attributeKindApi.update(kind.KindID, { kind_id: kind.KindID, key: kind.Key });
            });
            keyCell.appendChild(keyInput);

            const descCell = document.createElement('td');
            const descInput = document.createElement('input');
            descInput.type = 'text';
            descInput.value = kind.Description;
            descInput.addEventListener('change', () => {
                kind.Description = descInput.value;
                void attributeKindApi.update(kind.KindID, { kind_id: kind.KindID, description: kind.Description });
            });
            descCell.appendChild(descInput);

            const typeCell = document.createElement('td');
            const typeSelect = buildBaseTypeSelect();
            typeSelect.value = kind.BaseType;
            typeSelect.addEventListener('change', () => {
                kind.BaseType = typeSelect.value as AttributeBaseType;
                void attributeKindApi.update(kind.KindID, { kind_id: kind.KindID, base_type: kind.BaseType });
            });
            typeCell.appendChild(typeSelect);

            const actionsCell = document.createElement('td');
            const deleteButton = document.createElement('button');
            deleteButton.type = 'button';
            deleteButton.textContent = 'Delete';
            deleteButton.addEventListener('click', () => onDelete(kind));
            actionsCell.appendChild(deleteButton);

            row.append(keyCell, descCell, typeCell, actionsCell);
        }

        tbody.appendChild(row);
    });

    table.appendChild(tbody);
    return table;
}

function buildBaseTypeSelect(): HTMLSelectElement {
    const select = document.createElement('select');
    select.name = 'base_type';
    AttributeBaseTypesArray.forEach((type) => {
        const option = document.createElement('option');
        option.value = type;
        option.textContent = type.charAt(0).toUpperCase() + type.slice(1);
        select.appendChild(option);
    });
    return select;
}

function defaultConfigForType(base_type: string): AttributeConfig {
    switch (base_type) {
        case 'integer':
        case 'decimal':
            return {};
        case 'text':
            return {};
        case 'dropdown':
            return { values: ['todo', 'working', 'complete'] };
        case 'list':
            return { list_type: 'text' };
        default:
            return {};
    }
}

async function getErrorMessage(error: unknown, fallback: string): Promise<string> {
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
    return error instanceof Error && error.message ? error.message : fallback;
}

if (!customElements.get('attr-kind-settings-screen')) {
    customElements.define('attr-kind-settings-screen', AttrKindSettingsScreen);
}
