import API from "../api/api";
import ItemAPI from "../api/item";
import AddIcon from "../assets/icon/add.svg";
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



let refresh_value_input = (key: string, value_input: HTMLInputElement) => {
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

class AttributesView extends HTMLElement {
    private item_id: number = 0;
    private item_attributes: any = {};
    private container: HTMLElement | null = null;

    setup(item_id: number, attributes: object): AttributesView {
        this.item_id = item_id;
        this.item_attributes = attributes;

        this.refresh()
        return this;
    }

    async refresh() {
        this.innerHTML = "<div>Types</div><chips-input id='types_input'></chips-input>"; 
        let item_types_input = this.querySelector('#types_input')! as ChipsInput;
        
        if (attribute_kinds == null) {
            attribute_kinds = await API.item.AttributeKinds.get_all();
        }
        // Add all attribute kinds as a datalist for suggestions
        let datalistHtml = '<datalist id="attribute_kind_suggestions">'
        attribute_kinds?.forEach(kind => {
            datalistHtml += `<option value="${kind.key}">`
        });
        datalistHtml += "</datalist>"

        // For each attribute, make an input for it
        for (const [key, value] of Object.entries(this.item_attributes)) {
            this.add_key_value_input(key, value);
        }

        let container = document.createElement('form')
        container.classList.add('attribute')

        let key_input = document.createElement('input');
        key_input.id = "add_key_attribute"
        key_input.setAttribute('list', 'attribute_kind_suggestions');
        key_input.type = 'text';
        let value_input = document.createElement('input');

        key_input.addEventListener('input', () => {
            refresh_value_input(key_input.value, value_input);
        })

        container.addEventListener('submit', async (e: SubmitEvent) => {
            e.preventDefault();
            if (key_input.value == "" || value_input.value == "") {
                return;
            }
            try {
                await ItemAPI.Attributes.set_value(this.item_id, key_input.value, value_input.value);
                this.item_attributes[key_input.value] = value_input.value;
                this.refresh();
                (document.querySelector('#add_key_attribute') as HTMLInputElement)?.focus();
            } catch(e) {
                console.error(e)
            }
        })

        //Submit
        let submit = document.createElement('button');
        submit.innerHTML = `<img src="${AddIcon}" style="width: 20px !important">`;
        submit.type = 'submit';


        container.appendChild(key_input);
        container.appendChild(value_input);
        container.appendChild(submit)
        this.appendChild(container);
        
        item_types_input.set_value()
    }

    private add_key_value_input(key: string, value: any) {
        let container = document.createElement('div');
        container.classList.add('attribute');

        let key_input = document.createElement('input');
        let last_key = key;
        let attribute_kind = get_kind_for_key(key);


        let value_input = document.createElement('input');

        refresh_value_input(key, value_input);
        value_input.value = value as string;
        if (attribute_kind && attribute_kind.base_type == 'date') {
            value_input.value = (value as string ).substring(0, 10)
        }
        value_input.addEventListener('change', async () => {
            try {
                await ItemAPI.Attributes.set_value(this.item_id,
                    key_input.value,
                    value_input.value
                )
            } catch (e) {
                console.error(e)
            }
        })

        key_input.value = key;
        key_input.addEventListener('change', async () => {
            try {
                await ItemAPI.Attributes.rename(this.item_id, 
                    last_key,
                    key_input.value
                );
                last_key = key_input.value;
                this.item_attributes[key_input.value] = value_input;
                refresh_value_input(key_input.value, value_input);
            } catch (e) {
                console.error(e)
            }
        });
        
        let submit = document.createElement('button');
        submit.innerHTML = `<img src="${SettingsIcon}" style="width: 20px !important">`;
        submit.addEventListener('click', async () => {
            await ItemAPI.Attributes.remove(this.item_id, key_input.value);
            delete this.item_attributes[key_input.value];
            this.refresh();
        })
        
        container.appendChild(key_input);
        container.appendChild(value_input);
        container.appendChild(submit)
        this.appendChild(container);
    }

    // connectedCallback() {

    // }

    // disconnectedCallback() {

    // }
}

customElements.define('attributes-view', AttributesView)

export default AttributesView;