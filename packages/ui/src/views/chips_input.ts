import {icons} from '@zealot/content'
import "./chips-input.scss";

type ChipsAddEvent = CustomEvent<{items: string[]}>;
type ChipsRemoveEvent = CustomEvent<{items: string[]}>;

declare global {
    interface HTMLElementEventMap {
        "chips-add": ChipsAddEvent,
        "chips-remove": ChipsRemoveEvent
    }
}


class ChipsInput extends HTMLElement {
    private items_container!: HTMLDivElement;
    private items: string[] = [];
    private input!: HTMLInputElement;
    private enforce_unique: boolean = true;
    private on_click_item: ((item: string) => void) | null = null;
    private _inputType: 'text' | 'number' | 'date' = 'text';

    public set OnClickItem(value: (item: string) => void) {
        this.on_click_item = value;
        let views = this.querySelectorAll('.chip_item')!
        views.forEach((v) => {
            v.addEventListener('click', () => {
                this.on_click_item!((v as HTMLElement).textContent)
            })
        })
    }

    public get inputType(): 'text' | 'number' | 'date' {
        return this._inputType;
    }

    public set inputType(t: 'text' | 'number' | 'date') {
        this._inputType = t;
        if (this.isConnected) {
            this.refresh();
        }
    }

    public get value(): string[] {
        return this.items;
    }

    public set value(v: any) {
        if (Array.isArray(v)) {
            this.items = v;
        } else {
            this.items = [];
        }
        if (this.isConnected) {
            this.refresh();
        }
        this.dispatchEvent(new Event('change', {bubbles: true}));
    }

    connectedCallback() {
        this.refresh()
    }

    disconnectedCallback() {

    }

    private validate(item: string): boolean {
        if (this._inputType === 'number') {
            return item.trim() !== '' && !isNaN(parseFloat(item));
        }
        if (this._inputType === 'date') {
            return /^\d{4}-\d{2}-\d{2}$/.test(item.trim());
        }
        return true;
    }

    add_items(...items: string[]) {
        items = items.filter(item => this.validate(item));
        if (items.length === 0) return;

        if (this.enforce_unique) {
            items = items.filter(item => !this.items.includes(item));
            if (items.length == 0) {
                return;
            }
        }
        this.items.push(...items)
        this.refresh();

        this.dispatchEvent(new CustomEvent('chips-add', {
            detail: {items}
        }))
        this.dispatchEvent(new Event('change'));
    }

    remove_items(...items: string[]) {
        items.forEach(item => {
            let index = this.items.indexOf(item)
            if (index != -1) {
                this.items.splice(index, 1);
            }
        })
        this.refresh();

        this.dispatchEvent(new CustomEvent('chips-remove', {
            detail: {items}
        }));
        this.dispatchEvent(new Event('change'));
    }

    private refresh() {
        this.innerHTML = `<div type="items"></div><input type="${this._inputType}">`;
        this.items_container = this.querySelector('[type="items"]')!
        this.input = this.querySelector('input')!;

        this.input.addEventListener('keydown', (e: KeyboardEvent) => {
            if (e.key == "Enter") {
                if (this.input.value.length == 0) {
                    return;
                }
                e.preventDefault();
                e.stopPropagation();
                this.add_items(this.input.value)
                this.input.value = ""
                this.input.focus()
            }
        })

        this.addEventListener('click', () => {
            this.input.focus();
        })
        this.items_container.innerHTML = "";
        this.items.forEach(item => {
            let view = document.createElement("div");
            view.classList.add('chip_item')
            view.innerText = item;
            if (this.on_click_item) {
                view.addEventListener('click', () => {
                    this.on_click_item!(item);
                })
            }
            let delete_button = document.createElement("button");
            delete_button.innerHTML = `<img src="${icons.close}" style="display: inline;">`
            view.appendChild(delete_button)

            this.items_container.appendChild(view)
            setTimeout(() => {
                delete_button.addEventListener('click', (e: Event) => {
                    e.preventDefault();
                    this.remove_items(item)
                })}, 100)

        })
    }

    remove_item_by_index(index: number) {
        let item = this.items[index];
        this.items.splice(index, 1);
        this.refresh();
        this.dispatchEvent(new CustomEvent('chips-remove', {
            detail: {items: [item]}
        }));
        this.dispatchEvent(new Event('change'));
    }

    set_value(items: string[]) {
        this.add_items(...items)
    }

    get_value(): string[] {
        return this.items;
    }
}

customElements.define('chips-input', ChipsInput)

export default ChipsInput;
