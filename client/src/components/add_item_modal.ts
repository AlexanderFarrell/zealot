import API from "../api/api";
import ItemAPI, { type Item } from "../api/item";
import { get_item_types, get_type_for_name } from "../api/item_type";
import { router } from "../core/router";
import TypeSettingsScreen from "../screens/settings/type_settings_screen";
import { BaseElementEmpty, GenerateSelectHTML } from "./common/base_element";
import AttributeItemView, { AttributeValueView } from "./item/attribute_item_view";

class AddItemModal2 extends BaseElementEmpty {
    private attr_fields: Map<string, AttributeValueView> = new Map();

    async render() {
        this.classList.add('modal_background')

        let item_type_vals = ['', ...((await get_item_types()).map(i => i.name))];
        this.innerHTML = `
        <div class="inner_window">
            <label>Title</label>
            <input name="title" type="text" required>
            <label>Type</label>
            ${GenerateSelectHTML('item_type', '', item_type_vals)}
            <div name="additional_fields"></div>
            <button name="submit" style="margin-top: 5px">Create</button>
            <div name="error_message"></div>
        </div>
        ` 

        let inner_window = this.querySelector('.inner_window')! as HTMLDivElement;
        let title_input = this.querySelector('[name="title"]')! as HTMLInputElement;
        let type_select = this.querySelector('[name="item_type"]')! as HTMLSelectElement;
        let submit = this.querySelector('[name="submit"]')! as HTMLButtonElement;
        let error_message = this.querySelector('[name="error_message"]')! as HTMLDivElement;

        this.addEventListener('keydown', (e: KeyboardEvent) => {
            if (e.key == "Escape") {
                this.remove();
            }
        })

        // Handle clicking out to close, but not clicking inside modal        
        inner_window.addEventListener('click', (e: MouseEvent) => {
            e.stopPropagation();
        })
        this.addEventListener('click', (e: MouseEvent) => {
            this.remove();
        })

        title_input.addEventListener('keydown', (e: KeyboardEvent) => {
            console.log(e.key)
            if (e.key == "Enter") {
                submit.dispatchEvent(new Event('click'));
            }
        }) 

        type_select.addEventListener('change', () => {
            // Populate Additional Fields
            this.render_additional_fields();
        })

        this.render_additional_fields();

        submit.addEventListener('click', async () => {
            let attrs: any = {}
            let title = title_input.value;

            try {                 
                if (title_input.value == "") {
                    throw new Error("Please enter a title");
                }       
                this.attr_fields.forEach((field: AttributeValueView, key: string) => {
                    if (field.value == "") {
                        throw new Error("Please enter a value for " + key)
                    }
                    attrs[key] = field.value
                })
                let item: Item = {
                    item_id: -1,
                    title: title,
                    content: "",
                    attributes: attrs
                }
                item = await API.item.add(item);
                await API.item.assign_type(item.item_id, type_select.value);
                router.navigate(`/item/${encodeURIComponent(title)}`);
                this.remove();
                // if (await ItemAPI.add(item)) {
                //     router.navigate(`/item/${encodeURIComponent(title)}`)
                //     this.remove();
                // } else {
                //     throw new Error("Failed to add item")
                // }
            } catch (e) {
                // @ts-ignore
                error_message.innerText = e;
            }
        })

        title_input.focus();
    }

    async render_additional_fields() {
        let type_select = this.querySelector('[name="item_type"]')! as HTMLSelectElement;
        let additional_fields = this.querySelector('[name="additional_fields"]')! as HTMLDivElement;
        additional_fields.innerHTML = "";
        this.attr_fields = new Map();

        let item_type = await get_type_for_name(type_select.value);

        if (item_type == null) {
            return;
        }

        // additional_fields.innerHTML = type_select.value;
        item_type.required_attribute_keys?.forEach((key) => {
            let label = document.createElement('label');
            label.innerText = key

            let view = new AttributeValueView();
            view.init({
                key: key,
                value: ""
            })

            additional_fields.appendChild(label)
            additional_fields.appendChild(view)
            this.attr_fields.set(key, view);
        }) 
    }
}


// class AddItemModal extends HTMLElement {
//     public error_message: HTMLElement | null = null;

//     connectedCallback() {
//         this.classList.add('modal_background')

//         this.innerHTML = `
//         <div class="inner_window">
//             <input name="title" type="text">
//             ${GenerateSelectHTML('item_type', '', [])}
//             <div name="additional_fields"></div>
//         </div>
//         `

//         // Inner Window
//         let w = document.createElement('div');
//         w.classList.add('inner_window');
//         w.addEventListener('click', (e: MouseEvent) => {
//             e.stopPropagation();
//         })

//         this.addEventListener('click', (e: MouseEvent) => {
//             this.remove();
//         })

//         let title_input = document.createElement('input');
//         title_input.type = 'text';
//         title_input.placeholder = "Title";

//         let submit_button = document.createElement('button');
//         submit_button.innerText = "Create";
//         submit_button.addEventListener('click', () => {
//             this.create_new_item(title_input.value);
//         })
//         submit_button.style.marginTop = "5px";

//         this.error_message = document.createElement('div');
//         this.error_message.innerHTML = "";


//         this.addEventListener('keydown', (e: KeyboardEvent) => {
//             if (e.key == "Escape") {
//                 this.remove();
//             }

//             if (e.key == "Enter") {
//                 this.create_new_item(title_input.value);
//             }
//         })

//         this.appendChild(w);
//         w.appendChild(title_input);
//         w.appendChild(submit_button);
//         w.appendChild(this.error_message);

//         title_input.focus();

//     }

//     async create_new_item(title: string) {
//         if (title.length == 0) {
//             this.error_message!.innerHTML = "Please add a title";
//             return;
//         }

//         try {
//             let item = {
//                 item_id: -1,
//                 title: title,
//                 content: ""
//             }
//             if (await ItemAPI.add(item)) {
//                 router.navigate(`/item/${encodeURIComponent(title)}`)
//                 this.remove();
//             } else {
//                 this.error_message!.innerHTML = "Failed to add item";
//             }
//         } catch (e) {
//             console.error(e);
//             this.error_message!.innerHTML = "Error adding item";
//         }
//     }
// }

// customElements.define('add-item-modal', AddItemModal);
customElements.define('add-item-modal-two', AddItemModal2);

export default AddItemModal2;