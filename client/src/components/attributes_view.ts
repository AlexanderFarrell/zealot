import API from "../api/api";
import type { AttributeKind } from "../api/attribute_kind";
import ItemAPI, { type Item } from "../api/item";
import AddIcon from "../assets/icon/add.svg";
import DeleteIcon from "../assets/icon/close.svg";
import SettingsIcon from "../assets/icon/settings.svg";
import { events } from "../core/events";
import ChipsInput from "./common/chips_input";

// Store attribute kinds for validation
let attribute_kinds: null | Array<any> = null;
events.on('refresh_attributes', () => {
    attribute_kinds = null;
})

export function get_kind_for_key(key: string): any | undefined {
    return attribute_kinds?.find((e) => {
        return e.key == key;
    })
}


class AttributesView extends HTMLElement {
    private _item: Item | null = null;
    private container!: HTMLElement;

    public get item(): Item | null {
        return this._item;
    }

    public set item(value: Item) {
        this._item = value;
        this.refresh();
    }

    private async refresh() {
        // this.innerHTML = "<div>Types</div><chips-input id='types_input'></chips-input>"; 
        // let item_types_input = this.querySelector('#types_input')! as ChipsInput;

        this.innerHTML = `
        <datalist id="attribute_kind_suggestions">
        
        </datalist>
        <div name="attribute_container">
        </div>
        <form class="attribute">
            <input type="text" name="key" list="attribute_kind_suggestions" required>
            <input type="text" name="value" required>
            <button type="submit"><img src="${AddIcon}" style="width: 1em"></button>
        </form>
        `

        this.refresh_add_form();
        this.refresh_attribute_views();
    }
    
    private refresh_add_form() {
        // Datalist
        let suggestions = this.querySelector('#attribute_kind_suggestions')! as HTMLDataListElement;
        for (const [key, value] of Object.entries(this.item!.attributes!)) {
            suggestions.innerHTML += `<option value=${key}></option>`
        }

        // Form
        let form_add = this.querySelector('form')! as HTMLFormElement;
        let key_input = form_add.querySelector('input[name="key"]')! as HTMLInputElement;
        let value_input = form_add.querySelector('input[name="value"]')! as HTMLInputElement;

        key_input.addEventListener('input', () => {
            this.refresh_value_input(key_input.value, value_input);
        })

        form_add.addEventListener('submit', async (e: SubmitEvent) => {
            e.preventDefault();
            const data = new FormData(form_add);
            try {
                let key = data.get('key') as string;
                let value = data.get('value') as string;
                await ItemAPI.Attributes.set_value(this.item!.item_id, key, value)
                this.item!.attributes![key] = value;
                this.add_key_value_input(key, value);
                form_add.reset();
                (form_add.querySelector('[name="key"]') as HTMLInputElement)?.focus();
            } catch(e) {
                console.error(e)
            }
        })
    }

    private async refresh_attribute_views() {
        if (!attribute_kinds) {
            attribute_kinds = await API.item.AttributeKinds.get_all();
        }

        for (const [key, value] of Object.entries(this.item!.attributes!)) {
            this.add_key_value_input(key, value);
        }
    }

    private add_key_value_input(key: string, value: any) {
        let attributes_container = this.querySelector('[name="attribute_container"]')! as HTMLDivElement;
        let view = document.createElement('div');
        view.classList.add('attribute')
        view.innerHTML = `
            <input type="text" name="key" list="attribute_kind_suggestions" required>
            <input type="text" name="value" required>
            <button type="submit"><img src="${DeleteIcon}" style="width: 1em"></button>`;

        let key_input = view.querySelector('[name="key"]')! as HTMLInputElement;
        let value_input = view.querySelector('[name="value"]')! as HTMLInputElement;
        let button_submit = view.querySelector('[type="submit"]')! as HTMLButtonElement;

        // Set the value input to the type of the attribute kind.
        let kind = get_kind_for_key(key);
        this.refresh_value_input(key, value_input);

        // Set values
        key_input.value = key;
        this.set_value_input(value, value_input, kind);

        // Change listeners
        key_input.addEventListener('change', async () => {
            await ItemAPI.Attributes.rename(this.item!.item_id, key, key_input.value);
            delete this.item!.attributes![key];
            key = key_input.value;
            this.item!.attributes![key] = value_input.value;
            this.refresh_value_input(key, value_input);
        })
        value_input.addEventListener('change', async () => {
            await ItemAPI.Attributes.set_value(this.item!.item_id, key_input.value, value_input.value);
        })

        // Delete
        button_submit.addEventListener('click', async () => {
            await ItemAPI.Attributes.remove(this.item!.item_id, key_input.value);
            delete this.item!.attributes![key_input.value];
            this.refresh()
        })
        attributes_container.appendChild(view);
    }

    private refresh_value_input (key: string, value_input: HTMLInputElement) {
        let attribute_kind = get_kind_for_key(key);
        if (attribute_kind != undefined) {
            switch (attribute_kind.base_type) {
                case 'text':
                    value_input.type = 'text';
                    return;
                case 'integer':
                    value_input.type = 'number';
                    value_input.step = '1';
                    return;
                case 'decimal':
                    value_input.type = 'number';
                    return;
                case 'date':
                    value_input.type = 'date';
                    return;
                case 'week':
                    value_input.type = 'week';
                    return;
                case 'dropdown':
                    value_input.type = 'text';
                    return;
                case 'boolean':
                    value_input.type = 'checkbox';
                    return;
            }
        }
        value_input.type = "text";
    }

    private set_value_input(value: string, value_input: HTMLInputElement, kind?: AttributeKind) {
        if (kind && kind.base_type == 'date') {
            value_input.value = (value as string ).substring(0, 10)
        } else {
            value_input.value = value;
        }
    }
}

customElements.define('attributes-view', AttributesView)

export default AttributesView;