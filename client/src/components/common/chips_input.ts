import CloseIcon from "../../assets/icon/close.svg";

class ChipsInput extends HTMLElement {
    private items_container!: HTMLDivElement;
    private items: string[] = [];
    private input!: HTMLInputElement;
    private on_add_listeners: Set<(items: string[]) => void> = new Set();
    private on_remove_listeners: Set<(items: string[]) => void> = new Set();
    private enforce_unique: boolean = true;

    connectedCallback() {
        this.refresh()
    }

    disconnectedCallback() {

    }

    // Events
    public on_add(func: (items: string[]) => void) {
        this.on_add_listeners.add(func);
    }

    public on_remove(func: (items: string[]) => void) {
        this.on_remove_listeners.add(func)
    }

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

        this.on_add_listeners.forEach(listener => {
            listener(items)
        })
    }

    remove_items(...items: string[]) {
        items.forEach(item => {
            let index = this.items.indexOf(item)
            if (index != -1) {
                this.items.splice(index, 1);
            }
        })
        this.refresh();

        this.on_remove_listeners.forEach(listener => {
            listener(items)
        })
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
        this.on_remove_listeners.forEach(listener => {
            listener([item])
        })
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