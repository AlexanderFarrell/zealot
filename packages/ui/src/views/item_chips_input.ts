import { ItemAPI } from '@zealot/api/src/item';
import { getNavigator } from '@websoil/engine';
import type { Item } from '@zealot/domain/src/item';
import { icons } from '@zealot/content';
import { ItemSearchInline } from './item_search_inline';
import './chips-input.scss';

const itemApi = new ItemAPI('/api');

interface ItemChip {
    id: number;
    displayTitle: string;
}

/**
 * <item-chips-input>
 *
 * A list-of-item-IDs control. Each chip shows the item's DisplayTitle with a
 * close button; clicking a chip navigates to that item. An inline search input
 * lets the user add new items by searching.
 *
 * Stored value type: number[] (item IDs)
 *
 * Usage:
 *   const chips = new ItemChipsInput();
 *   chips.OnChange = (ids) => save(ids);
 *   container.appendChild(chips);
 *   chips.value = [1, 2, 3]; // set initial value
 */
export class ItemChipsInput extends HTMLElement {
    private _items: ItemChip[] = [];
    // Per-ID generation counters to discard stale GetById resolutions
    private _resolutionGen = 0;

    private _chipsContainer: HTMLDivElement | null = null;
    private _searchEl: ItemSearchInline | null = null;

    public OnChange: ((ids: number[]) => void) | null = null;
    public OnClickItem: ((id: number) => void) | null = null;

    get value(): number[] {
        return this._items.map(i => i.id);
    }

    set value(ids: number[]) {
        this._resolutionGen++;
        const gen = this._resolutionGen;

        // Immediately render placeholders
        this._items = ids.map(id => ({ id, displayTitle: `#${id}` }));
        this._refreshChips();

        // Resolve display titles asynchronously
        ids.forEach((id, index) => {
            void itemApi.GetById(id).then((item: Item) => {
                if (gen !== this._resolutionGen) return;
                const chip = this._items[index];
                if (!chip) return;
                chip.displayTitle = item.DisplayTitle;
                this._refreshChips();
            }).catch(() => {
                // Keep placeholder on failure
            });
        });
    }

    connectedCallback() {
        this._renderShell();
    }

    disconnectedCallback() {
        // Child components clean up their own listeners/timers.
    }

    private _renderShell() {
        this.innerHTML = `
            <div class="chips-input item-chips-input" style="position: relative;">
                <div class="item-chips-list" style="display: flex; flex-wrap: wrap; gap: 5px; align-items: center;"></div>
                <div class="item-chips-search" style="position: relative; display: inline-block; min-width: 160px; flex: 1 1 auto;">
                </div>
            </div>
        `;

        this._chipsContainer = this.querySelector('.item-chips-list');
        const searchHost = this.querySelector('.item-chips-search');
        const search = new ItemSearchInline();
        search.placeholder = 'Add item…';
        search.ResultsFilter = (items) => {
            const selectedIds = new Set(this._items.map((entry) => entry.id));
            return items.filter((item) => !selectedIds.has(item.ItemID));
        };
        search.OnSelect = (item) => {
            this._items.push({ id: item.ItemID, displayTitle: item.DisplayTitle });
            this._refreshChips();
            search.clear();
            this.OnChange?.(this.value);
            search.focus();
        };

        searchHost?.appendChild(search);
        this._searchEl = search;

        this._refreshChips();
    }

    private _refreshChips() {
        if (!this._chipsContainer) return;
        this._chipsContainer.innerHTML = '';

        this._items.forEach((item, index) => {
            const chip = document.createElement('div');
            chip.className = 'chip_item';

            const label = document.createElement('span');
            label.textContent = item.displayTitle;
            label.style.cursor = 'pointer';
            label.addEventListener('click', () => {
                if (this.OnClickItem) {
                    this.OnClickItem(item.id);
                } else {
                    getNavigator().openItemById(item.id);
                }
            });

            const delBtn = document.createElement('button');
            delBtn.type = 'button';
            delBtn.innerHTML = `<img src="${icons.close}" style="display: inline;">`;
            delBtn.addEventListener('click', (e) => {
                e.stopPropagation();
                this._items.splice(index, 1);
                this._refreshChips();
                this.OnChange?.(this.value);
            });

            chip.appendChild(label);
            chip.appendChild(delBtn);
            this._chipsContainer!.appendChild(chip);
        });
    }
}

if (!customElements.get('item-chips-input')) {
    customElements.define('item-chips-input', ItemChipsInput);
}
