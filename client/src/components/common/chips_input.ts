import CloseIcon from "../../assets/icon/close.svg";
import { router } from "../../core/router";

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
    // private on_add_listeners: Set<(items: string[]) => void> = new Set();
    // private on_remove_listeners: Set<(items: string[]) => void> = new Set();
    private enforce_unique: boolean = true;
    private on_click_item: ((item: string) => void) | null = null;

    public set OnClickItem(value: (item: string) => void) {

        this.on_click_item = value;
        let views = this.querySelectorAll('.chip_item')!
        views.forEach((v) => {
            v.addEventListener('click', () => {
                this.on_click_item!((v as HTMLElement).textContent)
            })
        })
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
        this.dispatchEvent(new Event('change', {bubbles: true}));
    }

    connectedCallback() {
        this.refresh()
    }

    disconnectedCallback() {

    }

    // // Events
    // public on_add(func: (items: string[]) => void) {
    //     this.on_add_listeners.add(func);
    // }

    // public on_remove(func: (items: string[]) => void) {
    //     this.on_remove_listeners.add(func)
    // }

    add_items(...items: string[]) {
        if (this.enforce_unique) {
            items = items.filter(item => {
                return !(item in this.items);
            })
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
        this.innerHTML = `<div type="items"></div><input type="text">`;
        this.items_container = this.querySelector('[type="items"]')!
        this.input = this.querySelector('[type="text"]')!;

        this.input.addEventListener('keydown', (e: KeyboardEvent) => {
            if (e.key == "Enter") {
                e.preventDefault();
                if (this.input.value.length == 0) {
                    return;
                }
                this.add_items(this.input.value)
                this.input.value = ""
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
            delete_button.innerHTML = `<img src="${CloseIcon}" style="display: inline;">`
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