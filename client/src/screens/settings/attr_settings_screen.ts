
import API from "../../api/api";
import DeleteIcon from "../../assets/icon/delete.svg";
import { events } from "../../core/events";
import AddIcon from "../../assets/icon/add.svg"

let selectHTML = `<select name="base_type" value="text">
                <option value="text">Text</option>
                <option value="integer">Integer</option>
                <option value="decimal">Decimal</option>
                <option value="date">Date</option>
                <option value="week">Week</option>
                <option value="dropdown">Dropdown</option>
                <option value="boolean">Boolean</option>
            </select>`

class AttributeSettingsScreen extends HTMLElement {
    connectedCallback() {
        this.innerHTML = `
        <h1>Attribute Kinds</h1>
        <form id='add_attribute_kind' class="box" style="display: grid; grid-template-columns: 1fr 2fr auto auto; margin-bottom: 10px">
            <label>Name:</label>
            <label>Description</label>
            <label>Kind</label>
            <div>&nbsp;</div>
            <input type="text" name="key">
            <input type="text" name="description">
            ${selectHTML}
            <button type="submit"><img src="${AddIcon}" style="width: 1em;"></button>
        </form>
        <div id='attribute_kinds_container'></div>`;

        (this.querySelector('#add_attribute_kind') as HTMLFormElement)!.addEventListener('submit', (e: SubmitEvent) => {
            e.preventDefault();
            const data = new FormData(this.querySelector('#add_attribute_kind')! as HTMLFormElement);
            API.item.AttributeKinds.add({
                key: data.get('key') as string,
                description: data.get('description') as string,
                base_type: data.get('base_type') as string,
                config: {}
            })
                .then(() => {
                    this.refresh();
                });
        });
        this.refresh();
    }

    async refresh() {
        let attribute_kinds_container = this.querySelector('#attribute_kinds_container')!;
        try {
            let attribute_kinds = await API.item.AttributeKinds.get_all();
            attribute_kinds_container.innerHTML = "";
            attribute_kinds.forEach((attribute_kind: any) => {
                let view = document.createElement('div');
                view.classList.add('attribute_kind')
                view.innerHTML = 
                `<input name="key" value="${attribute_kind['key']}">
                <input name="description" value="${attribute_kind['description']}">
                ${selectHTML}
                <button class="delete_button"><img style="width: 1em" src="${DeleteIcon}"></button>
                `;
                let key_input = view.querySelector('[name="key"]')! as HTMLInputElement;
                let description_input = view.querySelector('[name="description"]')! as HTMLInputElement;
                let base_type_input = view.querySelector('[name="base_type"]')! as HTMLSelectElement;
                let delete_button = view.querySelector('.delete_button')! as HTMLButtonElement;

                base_type_input.value = attribute_kind.base_type;
                if (attribute_kind.is_system) {
                    key_input.disabled = true;
                    description_input.disabled = true;
                    base_type_input.disabled = true;
                    delete_button.disabled = true;
                } else {
                    key_input.addEventListener('change', () => {
                        attribute_kind.key = (view.querySelector('[name="key"]')! as HTMLInputElement).value;
                        API.item.AttributeKinds.update(attribute_kind.kind_id, {key: attribute_kind.key})
                        events.emit('refresh_attributes')
                    })
                    description_input.addEventListener('change', () => {
                        attribute_kind.description = (view.querySelector('[name="description"]')! as 
                        HTMLInputElement).value;
                        API.item.AttributeKinds.update(attribute_kind.kind_id, {description: attribute_kind.description})
                        events.emit('refresh_attributes')
                    })
                    base_type_input.addEventListener('change', () => {
                        attribute_kind.base_type_input = base_type_input.value;
                        API.item.AttributeKinds.update(attribute_kind.kind_id, {base_type: attribute_kind.base_type})
                        events.emit('refresh_attributes')
                    })
                    delete_button.addEventListener('click', () => {
                        API.item.AttributeKinds.remove(attribute_kind.kind_id)
                            .then(() => {
                                events.emit('refresh_attributes')
                                view.remove();
                            })
                    })
                }

                attribute_kinds_container.appendChild(view)
            })
        }
        catch (e) {
            console.error(e)
        }
    }
}

customElements.define('attr-settings-screen', AttributeSettingsScreen)
export default AttributeSettingsScreen;