import { ItemAPI } from '@zealot/api/src/item';
import type { Item } from '@zealot/domain/src/item';

const itemApi = new ItemAPI('/api');
let nextListboxId = 0;

declare global {
    interface HTMLElementEventMap {
        'item-selected': CustomEvent<{ item: Item }>;
    }
}

export class ItemSearchInline extends HTMLElement {
    private inputEl: HTMLInputElement | null = null;
    private resultsEl: HTMLDivElement | null = null;
    private debounceTimer: ReturnType<typeof setTimeout> | null = null;
    private blurTimer: ReturnType<typeof setTimeout> | null = null;
    private readonly listboxId = `item-search-inline-results-${nextListboxId++}`;
    private requestId = 0;
    private activeIndex = -1;
    private rendered = false;
    private results: Item[] = [];
    private selectedItem: Item | null = null;

    public OnSelect: ((item: Item) => void) | null = null;
    public ResultsFilter: ((items: Item[]) => Item[]) | null = null;

    get value(): Item | null {
        return this.selectedItem;
    }

    set value(item: Item | null) {
        this.selectedItem = item;
        if (this.inputEl) {
            this.inputEl.value = item?.DisplayTitle ?? '';
        }
        this.closeDropdown();
    }

    get placeholder(): string {
        return this.getAttribute('placeholder') ?? 'Search item…';
    }

    set placeholder(value: string) {
        this.setAttribute('placeholder', value);
        if (this.inputEl) {
            this.inputEl.placeholder = value;
        }
    }

    connectedCallback(): void {
        if (!this.rendered) {
            this.render();
        }
    }

    disconnectedCallback(): void {
        this.clearTimers();
    }

    focus(): void {
        this.inputEl?.focus();
        this.inputEl?.select();
    }

    clear(): void {
        this.requestId += 1;
        this.selectedItem = null;
        this.results = [];
        this.activeIndex = -1;
        this.clearTimers();

        if (this.inputEl) {
            this.inputEl.value = '';
        }

        if (this.resultsEl) {
            this.resultsEl.innerHTML = '';
        }

        this.closeDropdown();
    }

    private render(): void {
        this.rendered = true;
        this.classList.add('item-search-inline');
        this.innerHTML = `
        <div class="item-search-inline-shell">
            <input
                type="search"
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
                spellcheck="false"
                role="combobox"
                aria-autocomplete="list"
                aria-controls="${this.listboxId}"
                aria-expanded="false"
            >
            <div id="${this.listboxId}" class="item-search-inline-results" role="listbox" hidden></div>
        </div>
        `;

        this.inputEl = this.querySelector('input');
        this.resultsEl = this.querySelector('.item-search-inline-results');

        if (!this.inputEl || !this.resultsEl) {
            return;
        }

        this.inputEl.placeholder = this.placeholder;
        this.inputEl.value = this.selectedItem?.DisplayTitle ?? '';

        this.inputEl.addEventListener('input', () => this.handleInput());
        this.inputEl.addEventListener('keydown', (event) => this.handleKeydown(event));
        this.inputEl.addEventListener('focus', () => {
            const hasContent = (this.inputEl?.value.trim().length ?? 0) > 0;
            if (hasContent && this.resultsEl && this.resultsEl.childElementCount > 0) {
                this.openDropdown();
            }
        });
        this.inputEl.addEventListener('blur', () => {
            if (this.blurTimer) {
                clearTimeout(this.blurTimer);
            }
            this.blurTimer = setTimeout(() => {
                if (!this.contains(document.activeElement)) {
                    this.closeDropdown();
                }
            }, 120);
        });
    }

    private clearTimers(): void {
        if (this.debounceTimer) {
            clearTimeout(this.debounceTimer);
            this.debounceTimer = null;
        }
        if (this.blurTimer) {
            clearTimeout(this.blurTimer);
            this.blurTimer = null;
        }
    }

    private handleInput(): void {
        if (!this.inputEl) {
            return;
        }

        if (this.selectedItem && this.inputEl.value !== this.selectedItem.DisplayTitle) {
            this.selectedItem = null;
        }

        this.requestId += 1;
        this.activeIndex = -1;

        if (this.debounceTimer) {
            clearTimeout(this.debounceTimer);
            this.debounceTimer = null;
        }

        const term = this.inputEl.value.trim();
        if (!term) {
            this.results = [];
            if (this.resultsEl) {
                this.resultsEl.innerHTML = '';
            }
            this.closeDropdown();
            return;
        }

        const pendingRequestId = this.requestId;
        this.showStatus('Searching…');
        this.debounceTimer = setTimeout(() => {
            void this.performSearch(term, pendingRequestId);
        }, 300);
    }

    private handleKeydown(event: KeyboardEvent): void {
        if (event.key === 'ArrowDown') {
            if (this.results.length === 0) {
                return;
            }
            event.preventDefault();
            this.moveActiveIndex(1);
            return;
        }

        if (event.key === 'ArrowUp') {
            if (this.results.length === 0) {
                return;
            }
            event.preventDefault();
            this.moveActiveIndex(-1);
            return;
        }

        if (event.key === 'Enter') {
            if (this.results.length === 0 || this.activeIndex < 0) {
                return;
            }
            event.preventDefault();
            const item = this.results[this.activeIndex];
            if (item) {
                this.selectItem(item);
            }
            return;
        }

        if (event.key === 'Escape' && !this.resultsEl?.hidden) {
            event.preventDefault();
            event.stopPropagation();
            this.closeDropdown();
        }
    }

    private async performSearch(term: string, requestId: number): Promise<void> {
        try {
            let items = await itemApi.Search(term);
            if (requestId !== this.requestId) {
                return;
            }

            if (this.ResultsFilter) {
                items = this.ResultsFilter(items);
            }

            this.results = items;
            this.activeIndex = items.length > 0 ? 0 : -1;

            if (items.length === 0) {
                this.showStatus('No results.');
                return;
            }

            this.renderResults();
        } catch {
            if (requestId !== this.requestId) {
                return;
            }
            this.showStatus('Search failed.', 'tool-error');
        }
    }

    private renderResults(): void {
        if (!this.resultsEl) {
            return;
        }

        this.resultsEl.innerHTML = '';

        this.results.forEach((item, index) => {
            const row = document.createElement('button');
            row.type = 'button';
            row.id = `${this.listboxId}-option-${index}`;
            row.className = 'item-search-inline-row';
            row.setAttribute('role', 'option');
            row.setAttribute('aria-selected', String(index === this.activeIndex));
            row.addEventListener('mouseenter', () => {
                this.activeIndex = index;
                this.updateActiveState();
            });
            row.addEventListener('mousedown', (event) => {
                event.preventDefault();
            });
            row.addEventListener('click', () => {
                this.selectItem(item);
            });

            const title = document.createElement('span');
            title.className = 'item-search-inline-row-title';
            title.textContent = item.DisplayTitle;
            row.appendChild(title);

            if (item.Types.length > 0) {
                const badges = document.createElement('span');
                badges.className = 'item-search-inline-row-badges';
                item.Types.forEach((typeRef) => {
                    const badge = document.createElement('span');
                    badge.className = 'tool-badge';
                    badge.textContent = typeRef.Name;
                    badges.appendChild(badge);
                });
                row.appendChild(badges);
            }

            this.resultsEl?.appendChild(row);
        });

        this.updateActiveState();
        this.openDropdown();
    }

    private showStatus(message: string, className = 'tool-muted'): void {
        if (!this.resultsEl) {
            return;
        }

        this.results = [];
        this.activeIndex = -1;
        this.resultsEl.innerHTML = `<p class="item-search-inline-status ${className}">${message}</p>`;
        this.updateActiveState();
        this.openDropdown();
    }

    private moveActiveIndex(delta: number): void {
        if (this.results.length === 0) {
            this.activeIndex = -1;
            this.updateActiveState();
            return;
        }

        this.activeIndex += delta;
        if (this.activeIndex < 0) {
            this.activeIndex = 0;
        }
        if (this.activeIndex >= this.results.length) {
            this.activeIndex = this.results.length - 1;
        }

        this.updateActiveState();
        this.openDropdown();
    }

    private updateActiveState(): void {
        const options = this.resultsEl?.querySelectorAll<HTMLButtonElement>('.item-search-inline-row') ?? [];
        options.forEach((option, index) => {
            const active = index === this.activeIndex;
            option.classList.toggle('is-active', active);
            option.setAttribute('aria-selected', String(active));
        });

        if (!this.inputEl) {
            return;
        }

        if (this.activeIndex >= 0) {
            this.inputEl.setAttribute('aria-activedescendant', `${this.listboxId}-option-${this.activeIndex}`);
        } else {
            this.inputEl.removeAttribute('aria-activedescendant');
        }
    }

    private selectItem(item: Item): void {
        this.requestId += 1;
        this.selectedItem = item;
        this.clearTimers();

        if (this.inputEl) {
            this.inputEl.value = item.DisplayTitle;
        }

        this.closeDropdown();
        this.OnSelect?.(item);
        this.dispatchEvent(new CustomEvent('item-selected', {
            bubbles: true,
            detail: { item },
        }));
    }

    private openDropdown(): void {
        if (!this.resultsEl || !this.inputEl) {
            return;
        }

        this.resultsEl.hidden = false;
        this.inputEl.setAttribute('aria-expanded', 'true');
    }

    private closeDropdown(): void {
        if (!this.resultsEl || !this.inputEl) {
            return;
        }

        this.resultsEl.hidden = true;
        this.inputEl.setAttribute('aria-expanded', 'false');
        this.inputEl.removeAttribute('aria-activedescendant');
    }
}

if (!customElements.get('item-search-inline')) {
    customElements.define('item-search-inline', ItemSearchInline);
}
