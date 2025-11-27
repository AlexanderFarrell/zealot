import ItemAPI from "../api/item";
import AddIcon from "../assets/icon/add.svg";
import SettingsIcon from "../assets/icon/settings.svg";

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

    refresh() {
        this.innerHTML = "";
        // For each attribute, make an input for it
        for (const [key, value] of Object.entries(this.item_attributes)) {
            this.add_key_value_input(key, value);
        }

        let container = document.createElement('form')
        container.classList.add('attribute')

        let key_input = document.createElement('input');
        key_input.type = 'text';
        let value_input = document.createElement('input');
        key_input.type = 'text';

        container.addEventListener('submit', async (e: SubmitEvent) => {
            e.preventDefault();
            try {
                await ItemAPI.Attributes.set_value(this.item_id, key_input.value, value_input.value);
                this.item_attributes[key_input.value] = value_input.value;
                this.refresh();
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
    }

    private add_key_value_input(key: string, value: any) {
        let container = document.createElement('div');
        container.classList.add('attribute');

        let key_input = document.createElement('input');
        let last_key = key;

        let value_input = document.createElement('input');
        value_input.value = value as string;
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
            } catch (e) {
                console.error(e)
            }
        });
        
        let submit = document.createElement('button');
        submit.innerHTML = `<img src="${SettingsIcon}" style="width: 20px !important">`;
        
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