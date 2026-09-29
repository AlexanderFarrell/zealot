import type { ItemAPI, SearchResult, SearchScope } from '@zealot/api/src/item';
import { getNavigator, openInNewTab, registerDropZone, unregisterDropZonesIn, registerContextMenu, unregisterContextMenuIn, Popups } from '@websoil/engine';
import { AttributeAPI } from '@zealot/api/src/attribute';
import { createItemTitleElement } from '../views/item_title';

const attrApi = new AttributeAPI('/api');

const PAGE_SIZE = 20;

interface SearchToolViewOptions {
    itemApi: ItemAPI;
}

export class SearchToolView extends HTMLElement {
    private itemApi: ItemAPI | null = null;
    private inputEl: HTMLInputElement | null = null;
    private resultsEl: HTMLDivElement | null = null;
    private loadMoreBtn: HTMLButtonElement | null = null;
    private debounceTimer: ReturnType<typeof setTimeout> | null = null;
    private requestId = 0;
    private rendered = false;
    private scope: SearchScope = 'title';
    private currentTerm = '';
    private currentOffset = 0;
    private hasMore = false;

    init(options: SearchToolViewOptions): this {
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
        if (this.debounceTimer) {
            clearTimeout(this.debounceTimer);
            this.debounceTimer = null;
        }
        unregisterDropZonesIn(this);
        unregisterContextMenuIn(this);
    }

    private clearResults(): void {
        if (!this.resultsEl) return;
        unregisterDropZonesIn(this.resultsEl);
        unregisterContextMenuIn(this.resultsEl);
        this.resultsEl.innerHTML = '';
    }

    focusInput(): void {
        this.inputEl?.focus();
        this.inputEl?.select();
    }

    private render(): void {
        if (!this.itemApi) {
            return;
        }

        this.rendered = true;
        this.innerHTML = `
        <div class="tool-panel">
            <div class="tool-panel-header">
                <h2>Search</h2>
            </div>
            <label class="tool-field">
                <span class="tool-label">Find items</span>
                <input type="search" placeholder="Search…">
            </label>
            <div class="search-scope-toggles">
                <button type="button" class="search-scope-btn is-active" data-scope="title">Title</button>
                <button type="button" class="search-scope-btn" data-scope="content">Content</button>
                <button type="button" class="search-scope-btn" data-scope="heading">Heading</button>
            </div>
            <div class="search-tool-results"></div>
        </div>
        `;

        this.inputEl = this.querySelector('input');
        this.resultsEl = this.querySelector('.search-tool-results');

        this.inputEl?.addEventListener('input', () => this.scheduleSearch());

        this.querySelectorAll<HTMLButtonElement>('.search-scope-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                const s = btn.dataset['scope'] as SearchScope;
                if (s) this.setScope(s);
            });
        });

        this.showPrompt();
    }

    private setScope(scope: SearchScope): void {
        this.scope = scope;
        this.querySelectorAll<HTMLButtonElement>('.search-scope-btn').forEach(btn => {
            btn.classList.toggle('is-active', btn.dataset['scope'] === scope);
        });
        if (this.currentTerm) {
            this.currentOffset = 0;
            void this.performSearch(this.currentTerm, this.requestId);
        }
    }

    private scheduleSearch(): void {
        const raw = this.inputEl?.value ?? '';

        if (raw.startsWith('#')) {
            const strippedTerm = raw.slice(1).trim();
            if (this.scope !== 'heading') {
                this.scope = 'heading';
                this.querySelectorAll<HTMLButtonElement>('.search-scope-btn').forEach(btn => {
                    btn.classList.toggle('is-active', btn.dataset['scope'] === 'heading');
                });
            }
            if (!strippedTerm) {
                this.showPrompt();
                return;
            }
            this.currentTerm = strippedTerm;
        } else {
            this.currentTerm = raw.trim();
        }

        this.requestId += 1;
        this.currentOffset = 0;

        if (this.debounceTimer) {
            clearTimeout(this.debounceTimer);
            this.debounceTimer = null;
        }

        if (!this.currentTerm) {
            this.showPrompt();
            return;
        }

        const pendingRequestId = this.requestId;
        this.showLoading();
        this.debounceTimer = setTimeout(() => {
            void this.performSearch(this.currentTerm, pendingRequestId);
        }, 300);
    }

    private async performSearch(term: string, requestId: number): Promise<void> {
        try {
            const results = await this.itemApi!.Search(term, {
                scope: this.scope,
                limit: PAGE_SIZE,
                offset: this.currentOffset,
            });
            if (requestId !== this.requestId) {
                return;
            }
            if (this.currentOffset === 0) {
                this.showResults(results, term);
            } else {
                this.appendResults(results, term);
            }
            this.hasMore = results.length === PAGE_SIZE;
            this.syncLoadMoreButton();
        } catch (error) {
            console.error(error);
            if (requestId !== this.requestId) {
                return;
            }
            const msg = (error as Error).message ?? 'Failed to search items.';
            this.showError(msg);
        }
    }

    private showPrompt(): void {
        this.clearResults();
        if (!this.resultsEl) return;
        this.resultsEl.innerHTML = `<p class="tool-muted">Type to search items. Prefix with # to search headings.</p>`;
        this.loadMoreBtn = null;
    }

    private showLoading(): void {
        this.clearResults();
        if (!this.resultsEl) return;
        this.resultsEl.innerHTML = `<p class="tool-muted">Searching…</p>`;
        this.loadMoreBtn = null;
    }

    private showError(message: string): void {
        this.clearResults();
        if (!this.resultsEl) return;
        this.resultsEl.innerHTML = `<p class="tool-error">${message}</p>`;
        this.loadMoreBtn = null;
    }

    private showResults(results: SearchResult[], term: string): void {
        this.clearResults();
        if (!this.resultsEl) return;
        this.loadMoreBtn = null;

        if (results.length === 0) {
            this.resultsEl.innerHTML = `<p class="tool-muted">No items found.</p>`;
            return;
        }

        const list = document.createElement('div');
        list.className = 'search-tool-list';
        results.forEach(result => list.appendChild(this.buildResultRow(result, term)));
        this.resultsEl.appendChild(list);

        const loadMore = document.createElement('button');
        loadMore.type = 'button';
        loadMore.className = 'search-tool-load-more';
        loadMore.textContent = 'Load more';
        loadMore.hidden = true;
        loadMore.addEventListener('click', () => {
            this.currentOffset += PAGE_SIZE;
            this.requestId += 1;
            void this.performSearch(this.currentTerm, this.requestId);
        });
        this.resultsEl.appendChild(loadMore);
        this.loadMoreBtn = loadMore;
    }

    private appendResults(results: SearchResult[], term: string): void {
        if (!this.resultsEl) return;
        const list = this.resultsEl.querySelector<HTMLDivElement>('.search-tool-list');
        if (!list) return;
        results.forEach(result => list.appendChild(this.buildResultRow(result, term)));
    }

    private syncLoadMoreButton(): void {
        if (this.loadMoreBtn) {
            this.loadMoreBtn.hidden = !this.hasMore;
        }
    }

    private buildResultRow(result: SearchResult, _term: string): HTMLButtonElement {
        const { item } = result;
        const row = document.createElement('button');
        row.type = 'button';
        row.className = 'search-tool-result';
        row.draggable = true;

        row.appendChild(createItemTitleElement(item, { className: 'search-tool-result-title' }));

        if (result.snippet) {
            const snippet = document.createElement('span');
            snippet.className = 'search-tool-result-snippet';
            snippet.textContent = result.snippet;
            row.appendChild(snippet);
        }

        const meta = document.createElement('span');
        meta.className = 'search-tool-result-meta';

        const scopeBadge = document.createElement('span');
        scopeBadge.className = 'tool-badge search-scope-badge';
        scopeBadge.textContent = result.matchScope;
        meta.appendChild(scopeBadge);

        if (item.Types.length > 0) {
            item.Types.forEach((typeRef) => {
                const badge = document.createElement('span');
                badge.className = 'tool-badge';
                badge.textContent = typeRef.Name;
                meta.appendChild(badge);
            });
        }

        row.appendChild(meta);

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
            { label: 'Open in New Tab', onClick: () => openInNewTab(`/item/${encodeURIComponent(item.Title)}`) },
            { label: 'Open in New Window', onClick: () => window.open(`/item/${encodeURIComponent(item.Title)}`, '_blank', 'noopener,noreferrer') },
            { label: 'Copy Link', onClick: () => {
                void navigator.clipboard.writeText(`${window.location.origin}/item/${encodeURIComponent(item.Title)}`);
                Popups.add('Link copied');
            }},
        ]);

        return row;
    }
}

if (!customElements.get('search-tool-view')) {
    customElements.define('search-tool-view', SearchToolView);
}
