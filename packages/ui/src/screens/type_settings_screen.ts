import { BaseElementEmpty, getNavigator } from '@websoil/engine';
import { ItemTypeAPI } from '@zealot/api/src/item_type';
import type { ItemTypeSummary } from '@zealot/domain/src/item_type';
import { ConfirmDialog } from '../common/confirm_dialog';
import { LoadingSpinner } from '../common/loading_spinner';

const itemTypeApi = new ItemTypeAPI('/api');

export class TypeSettingsScreen extends BaseElementEmpty {
    private createError: string | null = null;
    private createName = '';
    private creating = false;
    private renderId = 0;

    async render() {
        const renderId = ++this.renderId;

        this.className = 'type-settings-screen';
        this.innerHTML = '';

        const shell = document.createElement('div');
        shell.className = 'type-settings-shell';

        const header = document.createElement('div');
        header.className = 'type-settings-header';

        const heading = document.createElement('h2');
        heading.textContent = 'Item Types';
        const copy = document.createElement('p');
        copy.className = 'tool-muted';
        copy.textContent = 'Create, inspect, and delete item types.';
        header.append(heading, copy);

        const form = document.createElement('form');
        form.className = 'type-settings-create';
        form.innerHTML = `
            <label class="tool-field">
                <span class="tool-label">New Type Name</span>
                <input name="type_name" type="text" placeholder="e.g. Project">
            </label>
            <button type="submit">Create Type</button>
        `;

        const input = form.querySelector<HTMLInputElement>('[name="type_name"]');
        if (input) {
            input.value = this.createName;
            input.disabled = this.creating;
            input.addEventListener('input', () => {
                this.createName = input.value;
            });
        }

        const submit = form.querySelector<HTMLButtonElement>('button[type="submit"]');
        if (submit) {
            submit.disabled = this.creating;
            submit.textContent = this.creating ? 'Creating…' : 'Create Type';
        }

        form.addEventListener('submit', (event) => {
            event.preventDefault();
            void this.createType();
        });

        shell.append(header, form);

        if (this.createError) {
            const error = document.createElement('p');
            error.className = 'tool-error';
            error.textContent = this.createError;
            shell.appendChild(error);
        }

        const list = document.createElement('div');
        list.className = 'type-settings-list';
        list.appendChild(new LoadingSpinner());
        shell.appendChild(list);

        this.appendChild(shell);

        try {
            const summaries = await itemTypeApi.get_summaries();
            if (renderId !== this.renderId) {
                return;
            }
            this.renderSummaries(list, summaries);
        } catch (error) {
            if (renderId !== this.renderId) {
                return;
            }
            list.innerHTML = '';
            const message = document.createElement('p');
            message.className = 'tool-error';
            message.textContent = error instanceof Error && error.message
                ? error.message
                : 'Failed to load item types.';
            list.appendChild(message);
        }
    }

    private renderSummaries(container: HTMLElement, summaries: ItemTypeSummary[]): void {
        container.innerHTML = '';

        if (summaries.length === 0) {
            const empty = document.createElement('p');
            empty.className = 'tool-muted';
            empty.textContent = 'No item types exist yet.';
            container.appendChild(empty);
            return;
        }

        const table = document.createElement('table');
        table.className = 'types-summary-table';
        table.innerHTML = `
            <thead>
                <tr>
                    <th>Name</th>
                    <th>Assigned Attributes</th>
                    <th>Items</th>
                    <th></th>
                </tr>
            </thead>
        `;

        const tbody = document.createElement('tbody');
        summaries.forEach((summary) => {
            const row = document.createElement('tr');

            const name = document.createElement('td');
            name.textContent = summary.Name;
            if (summary.IsSystem) {
                const badge = document.createElement('span');
                badge.className = 'tool-badge';
                badge.textContent = 'System';
                name.appendChild(badge);
            }

            const attrs = document.createElement('td');
            attrs.textContent = String(summary.RequiredAttributesCount);

            const items = document.createElement('td');
            items.textContent = String(summary.ItemCount);

            const actions = document.createElement('td');
            actions.className = 'type-settings-actions';

            const editButton = document.createElement('button');
            editButton.type = 'button';
            editButton.textContent = 'Edit';
            editButton.addEventListener('click', () => {
                getNavigator().openType(summary.Name);
            });

            const deleteButton = document.createElement('button');
            deleteButton.type = 'button';
            deleteButton.textContent = 'Delete';
            deleteButton.disabled = summary.IsSystem;
            deleteButton.addEventListener('click', () => {
                void this.deleteType(summary);
            });

            actions.append(editButton, deleteButton);
            row.append(name, attrs, items, actions);
            tbody.appendChild(row);
        });

        table.appendChild(tbody);
        container.appendChild(table);
    }

    private async createType(): Promise<void> {
        if (this.creating) {
            return;
        }

        const name = this.createName.trim();
        if (!name) {
            this.createError = 'Type name is required.';
            void this.render();
            return;
        }

        this.creating = true;
        this.createError = null;
        void this.render();

        try {
            const created = await itemTypeApi.create({
                description: '',
                name,
                required_attributes: [],
            });
            this.createName = '';
            getNavigator().openType(created.Name);
        } catch (error) {
            this.createError = await this.getErrorMessage(error, 'Failed to create item type.');
            this.creating = false;
            void this.render();
            return;
        }

        this.creating = false;
    }

    private async deleteType(summary: ItemTypeSummary): Promise<void> {
        const confirmed = await ConfirmDialog.show(`Delete the "${summary.Name}" item type?`);
        if (!confirmed) {
            return;
        }

        try {
            await itemTypeApi.remove(summary.TypeID);
            void this.render();
        } catch (error) {
            this.createError = await this.getErrorMessage(error, 'Failed to delete item type.');
            void this.render();
        }
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

        return error instanceof Error && error.message ? error.message : fallback;
    }
}

if (!customElements.get('type-settings-screen')) {
    customElements.define('type-settings-screen', TypeSettingsScreen);
}
