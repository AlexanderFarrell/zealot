import type { ItemAPI } from '@zealot/api/src/item';
import { getNavigator } from '@websoil/engine';
import type { Item } from '@zealot/domain/src/item';

interface SearchToolViewOptions {
    itemApi: ItemAPI;
}

export class SearchToolView extends HTMLElement {
    private itemApi: ItemAPI | null = null;
    private inputEl: HTMLInputElement | null = null;
    private resultsEl: HTMLDivElement | null = null;
    private debounceTimer: ReturnType<typeof setTimeout> | null = null;
    private requestId = 0;
    private rendered = false;

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
                <input type="search" placeholder="Search by title">
            </label>
            <div class="search-tool-results"></div>
        </div>
        `;

        this.inputEl = this.querySelector('input');
        this.resultsEl = this.querySelector('.search-tool-results');

        this.inputEl?.addEventListener('input', () => this.scheduleSearch());
        this.showPrompt();
    }

    private scheduleSearch(): void {
        const term = this.inputEl?.value.trim() ?? '';
        this.requestId += 1;

        if (this.debounceTimer) {
            clearTimeout(this.debounceTimer);
            this.debounceTimer = null;
        }

        if (!term) {
            this.showPrompt();
            return;
        }

        const pendingRequestId = this.requestId;
        this.showLoading();
        this.debounceTimer = setTimeout(() => {
            void this.performSearch(term, pendingRequestId);
        }, 300);
    }

    private async performSearch(term: string, requestId: number): Promise<void> {
        try {
            const results = await this.itemApi!.Search(term);
            if (requestId !== this.requestId) {
                return;
            }
            this.showResults(results);
        } catch (error) {
            console.error(error);
            if (requestId !== this.requestId) {
                return;
            }
            this.showError((error as Error).message ?? 'Failed to search items.');
        }
    }

    private showPrompt(): void {
        if (!this.resultsEl) {
            return;
        }
        this.resultsEl.innerHTML = `<p class="tool-muted">Type a title to search items.</p>`;
    }

    private showLoading(): void {
        if (!this.resultsEl) {
            return;
        }
        this.resultsEl.innerHTML = `<p class="tool-muted">Searching…</p>`;
    }

    private showError(message: string): void {
        if (!this.resultsEl) {
            return;
        }
        this.resultsEl.innerHTML = `<p class="tool-error">${message}</p>`;
    }

    private showResults(results: Item[]): void {
        if (!this.resultsEl) {
            return;
        }

        this.resultsEl.innerHTML = '';

        if (results.length === 0) {
            this.resultsEl.innerHTML = `<p class="tool-muted">No items found.</p>`;
            return;
        }

        const list = document.createElement('div');
        list.className = 'search-tool-list';

        results.forEach((item) => {
            const row = document.createElement('button');
            row.type = 'button';
            row.className = 'search-tool-result';

            const title = document.createElement('span');
            title.className = 'search-tool-result-title';
            title.textContent = item.DisplayTitle;
            row.appendChild(title);

            if (item.Types.length > 0) {
                const badges = document.createElement('span');
                badges.className = 'search-tool-result-badges';
                item.Types.forEach((typeRef) => {
                    const badge = document.createElement('span');
                    badge.className = 'tool-badge';
                    badge.textContent = typeRef.Name;
                    badges.appendChild(badge);
                });
                row.appendChild(badges);
            }

            row.addEventListener('click', () => {
                getNavigator().openItemById(item.ItemID);
            });

            list.appendChild(row);
        });

        this.resultsEl.appendChild(list);
    }
}

if (!customElements.get('search-tool-view')) {
    customElements.define('search-tool-view', SearchToolView);
}
