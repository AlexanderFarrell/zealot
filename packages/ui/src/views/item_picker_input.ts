import { ItemAPI } from '@zealot/api/src/item';
import { getNavigator } from '@websoil/engine';
import type { Item } from '@zealot/domain/src/item';
import { icons } from '@zealot/content';
import { ItemSearchInline } from './item_search_inline';
import './chips-input.scss';

const itemApi = new ItemAPI('/api');

/**
 * <item-picker-input>
 *
 * Single item-reference control. Stores a numeric item ID and displays the
 * item's DisplayTitle as a clickable chip. When no value is set, shows a
 * search input with a dropdown of results.
 *
 * Usage:
 *   const picker = new ItemPickerInput();
 *   picker.OnChange = (id) => console.log('selected', id);
 *   container.appendChild(picker);
 *   picker.value = 42; // set initial value
 */
export class ItemPickerInput extends HTMLElement {
    private _value: number | null = null;
    private _displayTitle: string = '';
    // Generation counter so stale GetById resolutions don't overwrite newer values
    private _generation = 0;

    public OnChange: ((itemId: number | null) => void) | null = null;

    get value(): number | null {
        return this._value;
    }

    set value(id: number | null) {
        this._value = id;
        this._displayTitle = id !== null ? `#${id}` : '';
        this._render();

        if (id !== null) {
            const gen = ++this._generation;
            void itemApi.GetById(id).then((item: Item) => {
                if (gen !== this._generation) return;
                this._displayTitle = item.DisplayTitle;
                this._render();
            }).catch(() => {
                // Keep placeholder if lookup fails
            });
        }
    }

    connectedCallback() {
        this._render();
    }

    disconnectedCallback() {
        // Nothing to clean up; child elements own their listeners/timers.
    }

    private _render() {
        if (!this.isConnected) return;

        this.innerHTML = '';
        this.style.display = 'block';
        this.style.position = 'relative';

        if (this._value !== null) {
            this._renderChip();
        } else {
            this._renderSearch();
        }
    }

    private _renderChip() {
        const chip = document.createElement('div');
        chip.className = 'chip_item';
        chip.style.cursor = 'pointer';

        const label = document.createElement('span');
        label.textContent = this._displayTitle;
        label.addEventListener('click', () => {
            if (this._value !== null) {
                getNavigator().openItemById(this._value);
            }
        });

        const clearBtn = document.createElement('button');
        clearBtn.type = 'button';
        clearBtn.innerHTML = `<img src="${icons.close}" style="display: inline;">`;
        clearBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            this._generation++;
            this._value = null;
            this._displayTitle = '';
            this._render();
            this.OnChange?.(null);
        });

        chip.appendChild(label);
        chip.appendChild(clearBtn);
        this.appendChild(chip);
    }

    private _renderSearch() {
        const search = new ItemSearchInline();
        search.placeholder = 'Search item…';
        search.OnSelect = (item: Item) => {
            this._generation++;
            this._value = item.ItemID;
            this._displayTitle = item.DisplayTitle;
            this._render();
            this.OnChange?.(item.ItemID);
        };
        this.appendChild(search);
    }
}

if (!customElements.get('item-picker-input')) {
    customElements.define('item-picker-input', ItemPickerInput);
}
