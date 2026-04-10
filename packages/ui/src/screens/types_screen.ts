import { BaseElementEmpty, getNavigator } from '@websoil/engine';
import { ItemTypeAPI } from '@zealot/api/src/item_type';
import type { ItemTypeSummary } from '@zealot/domain/src/item_type';
import { LoadingSpinner } from '../common/loading_spinner';

const itemTypeApi = new ItemTypeAPI('/api');

export class TypesScreen extends BaseElementEmpty {
    private renderId = 0;

    async render() {
        const renderId = ++this.renderId;

        this.className = 'types-screen';
        this.innerHTML = '';

        const header = document.createElement('div');
        header.className = 'types-screen-header';

        const titleGroup = document.createElement('div');
        const title = document.createElement('h1');
        title.textContent = 'Types';
        const subtitle = document.createElement('p');
        subtitle.className = 'tool-muted';
        subtitle.textContent = 'Browse item types and open a type to inspect its attributes and items.';
        titleGroup.append(title, subtitle);

        const createButton = document.createElement('button');
        createButton.type = 'button';
        createButton.textContent = 'Create Type';
        createButton.addEventListener('click', () => {
            getNavigator().openSettings('types');
        });

        header.append(titleGroup, createButton);
        this.appendChild(header);

        const body = document.createElement('div');
        body.className = 'types-screen-body';
        body.appendChild(new LoadingSpinner());
        this.appendChild(body);

        try {
            const summaries = await itemTypeApi.get_summaries();
            if (renderId !== this.renderId) {
                return;
            }
            this.renderSummaries(body, summaries);
        } catch (error) {
            if (renderId !== this.renderId) {
                return;
            }
            body.innerHTML = '';
            const message = document.createElement('p');
            message.className = 'tool-error';
            message.textContent = error instanceof Error && error.message
                ? error.message
                : 'Failed to load item types.';
            body.appendChild(message);
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
                    <th>Type</th>
                    <th>Assigned Attributes</th>
                    <th>Items</th>
                </tr>
            </thead>
        `;

        const tbody = document.createElement('tbody');
        summaries.forEach((summary) => {
            const row = document.createElement('tr');
            row.tabIndex = 0;
            row.addEventListener('click', () => {
                getNavigator().openType(summary.Name);
            });
            row.addEventListener('keydown', (event) => {
                if (event.key === 'Enter' || event.key === ' ') {
                    event.preventDefault();
                    getNavigator().openType(summary.Name);
                }
            });

            const name = document.createElement('td');
            const nameButton = document.createElement('button');
            nameButton.type = 'button';
            nameButton.className = 'type-row-link';
            nameButton.textContent = summary.Name;
            nameButton.addEventListener('click', (event) => {
                event.stopPropagation();
                getNavigator().openType(summary.Name);
            });
            name.appendChild(nameButton);
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

            row.append(name, attrs, items);
            tbody.appendChild(row);
        });

        table.appendChild(tbody);
        container.appendChild(table);
    }
}

customElements.define('types-screen', TypesScreen);
