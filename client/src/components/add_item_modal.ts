import ItemAPI from "../api/item";
import { events } from "../core/events";
import { router } from "../core/router";
import { switch_item_to } from "../screens/item_screen";


class AddItemModal extends HTMLElement {
    public error_message: HTMLElement | null = null;

    connectedCallback() {
        this.classList.add('modal_background')

        // Inner Window
        let w = document.createElement('div');
        w.classList.add('inner_window');
        w.addEventListener('click', (e: MouseEvent) => {
            e.stopPropagation();
        })

        this.addEventListener('click', (e: MouseEvent) => {
            this.remove();
        })

        let title_input = document.createElement('input');
        title_input.type = 'text';
        title_input.placeholder = "Title";

        let submit_button = document.createElement('button');
        submit_button.innerText = "Create";
        submit_button.addEventListener('click', () => {
            this.create_new_item(title_input.value);
        })
        submit_button.style.marginTop = "5px";

        this.error_message = document.createElement('div');
        this.error_message.innerHTML = "";


        this.addEventListener('keydown', (e: KeyboardEvent) => {
            if (e.key == "Escape") {
                this.remove();
            }

            if (e.key == "Enter") {
                this.create_new_item(title_input.value);
            }
        })

        this.appendChild(w);
        w.appendChild(title_input);
        w.appendChild(submit_button);
        w.appendChild(this.error_message);

        title_input.focus();

    }

    async create_new_item(title: string) {
        if (title.length == 0) {
            this.error_message!.innerHTML = "Please add a title";
            return;
        }

        try {
            if (await ItemAPI.add(title)) {
                router.navigate(`/item/${title}`)
                this.remove();
            } else {
                this.error_message!.innerHTML = "Failed to add item";
            }
        } catch (e) {
            console.error(e);
            this.error_message!.innerHTML = "Error adding item";
        }
    }
}

customElements.define('add-item-modal', AddItemModal);

export default AddItemModal;