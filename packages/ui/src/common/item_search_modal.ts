import { getNavigator } from '@websoil/engine';
import { ItemSearchInline } from '../views/item_search_inline';

export class ItemSearchModal extends HTMLElement {
    private searchEl: ItemSearchInline | null = null;
    private rendered = false;

    static show(): ItemSearchModal {
        const existing = document.querySelector('item-search-modal') as ItemSearchModal | null;
        if (existing) {
            existing.focusSearch();
            return existing;
        }

        const modal = new ItemSearchModal();
        (document.body ?? document.documentElement).appendChild(modal);
        queueMicrotask(() => modal.focusSearch());
        return modal;
    }

    connectedCallback(): void {
        if (!this.rendered) {
            this.render();
        }
    }

    private render(): void {
        this.rendered = true;
        this.classList.add('modal_background');
        this.classList.add('item-search-modal');
        this.innerHTML = `
        <div class="inner_window item-search-modal-window" role="dialog" aria-modal="true" aria-label="Search items">
            <div class="tool-panel">
                <div class="tool-panel-header">
                    <h2>Search</h2>
                </div>
                <div data-role="search"></div>
            </div>
        </div>
        `;

        const searchHost = this.querySelector<HTMLElement>('[data-role="search"]');
        const search = new ItemSearchInline();
        search.placeholder = 'Search items by title…';
        search.OnSelect = (item) => {
            getNavigator().openItemById(item.ItemID);
            this.close();
        };
        searchHost?.appendChild(search);
        this.searchEl = search;

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
    }

    private focusSearch(): void {
        window.requestAnimationFrame(() => {
            this.searchEl?.focus();
        });
    }

    private close(): void {
        this.remove();
    }
}

if (!customElements.get('item-search-modal')) {
    customElements.define('item-search-modal', ItemSearchModal);
}
