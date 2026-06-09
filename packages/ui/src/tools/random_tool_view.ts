import type { ItemAPI } from '@zealot/api/src/item';
import { Item } from '@zealot/domain/src/item';
import { getNavigator, NavigationCommands, commands, registerDropZone, unregisterDropZonesIn, registerContextMenu, unregisterContextMenuIn, Popups } from '@websoil/engine';
import { AttributeAPI } from '@zealot/api/src/attribute';
import { createItemTitleElement } from '../views/item_title';

const attrApi = new AttributeAPI('/api');

const RANDOM_COUNT = 15;

interface RandomToolViewOptions {
    itemApi: ItemAPI;
}

export class RandomToolView extends HTMLElement {
    private itemApi: ItemAPI | null = null;
    private resultsEl: HTMLDivElement | null = null;
    private rendered = false;

    init(options: RandomToolViewOptions): this {
        this.itemApi = options.itemApi;
        if (this.isConnected) {
            this.render();
        }
        return this;
    }

    connectedCallback(): void {
        if (!this.rendered) {
            this.render();
        }
    }

    disconnectedCallback(): void {
        unregisterDropZonesIn(this);
        unregisterContextMenuIn(this);
    }

    private clearResults(): void {
        if (!this.resultsEl) return;
        unregisterDropZonesIn(this.resultsEl);
        unregisterContextMenuIn(this.resultsEl);
        this.resultsEl.innerHTML = '';
    }

    private render(): void {
        if (!this.itemApi) return;
        this.rendered = true;

        this.innerHTML = `
        <div class="tool-panel">
            <div class="tool-panel-header">
                <h2>Random Items</h2>
            </div>
            <div class="random-tool-actions">
                <button type="button" class="random-tool-refresh-btn">Get ${RANDOM_COUNT} random items</button>
                <button type="button" class="random-tool-navigate-btn">Navigate to random</button>
            </div>
            <div class="random-tool-results"></div>
        </div>
        `;

        this.resultsEl = this.querySelector('.random-tool-results');

        this.querySelector('.random-tool-refresh-btn')?.addEventListener('click', () => {
            void this.fetchRandom();
        });
        this.querySelector('.random-tool-navigate-btn')?.addEventListener('click', () => {
            commands.runner.run(NavigationCommands.openRandomItem);
        });

        void this.fetchRandom();
    }

    private async fetchRandom(): Promise<void> {
        this.showLoading();
        try {
            const items = await this.itemApi!.GetRandom(RANDOM_COUNT);
            this.showResults(items);
        } catch (error) {
            const msg = (error as Error).message ?? 'Failed to load random items.';
            this.showError(msg);
        }
    }

    private showLoading(): void {
        this.clearResults();
        if (!this.resultsEl) return;
        this.resultsEl.innerHTML = `<p class="tool-muted">Loading…</p>`;
    }

    private showError(message: string): void {
        this.clearResults();
        if (!this.resultsEl) return;
        this.resultsEl.innerHTML = `<p class="tool-error">${message}</p>`;
    }

    private showResults(items: Item[]): void {
        this.clearResults();
        if (!this.resultsEl) return;

        if (items.length === 0) {
            this.resultsEl.innerHTML = `<p class="tool-muted">No items found.</p>`;
            return;
        }

        const list = document.createElement('div');
        list.className = 'search-tool-list';
        items.forEach(item => list.appendChild(this.buildResultRow(item)));
        this.resultsEl.appendChild(list);
    }

    private buildResultRow(item: Item): HTMLButtonElement {
        const row = document.createElement('button');
        row.type = 'button';
        row.className = 'search-tool-result';
        row.draggable = true;

        row.appendChild(createItemTitleElement(item, { className: 'search-tool-result-title' }));

        if (item.Types.length > 0) {
            const meta = document.createElement('span');
            meta.className = 'search-tool-result-meta';
            item.Types.forEach((typeRef) => {
                const badge = document.createElement('span');
                badge.className = 'tool-badge';
                badge.textContent = typeRef.Name;
                meta.appendChild(badge);
            });
            row.appendChild(meta);
        }

        row.addEventListener('click', () => {
            getNavigator().openItemById(item.ItemID);
        });
        row.addEventListener('dragstart', (e) => {
            e.dataTransfer?.setData('text/plain', String(item.ItemID));
            if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move';
        });
        registerDropZone({
            element: row,
            onDragOver: (e) => {
                if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
                row.classList.add('search-tool-result--drop-target');
            },
            onDrop: (draggedItemId) => {
                if (draggedItemId === item.ItemID) return;
                void attrApi.set_value(draggedItemId, 'Parent', [item.ItemID]);
            },
            onDragLeave: () => {
                row.classList.remove('search-tool-result--drop-target');
            },
        });
        registerContextMenu(row, () => [
            { label: 'Open', onClick: () => getNavigator().openItemById(item.ItemID) },
            { label: 'Open in New Tab', onClick: () => window.open(`/item/${encodeURIComponent(item.Title)}`, '_blank') },
            { label: 'Open in New Window', onClick: () => window.open(`/item/${encodeURIComponent(item.Title)}`, '_blank', 'noopener,noreferrer') },
            { label: 'Copy Link', onClick: () => {
                void navigator.clipboard.writeText(`${window.location.origin}/item/${encodeURIComponent(item.Title)}`);
                Popups.add('Link copied');
            }},
        ]);

        return row;
    }
}

if (!customElements.get('random-tool-view')) {
    customElements.define('random-tool-view', RandomToolView);
}
