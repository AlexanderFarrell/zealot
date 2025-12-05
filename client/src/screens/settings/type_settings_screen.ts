import API from "../../api/api";
import AuthAPI from "../../api/auth";
import { events } from "../../core/events";
import { get_settings } from "../../core/settings";
import DeleteIcon from "../../assets/icon/delete.svg";

let selectHTML = `<select name="base_type" value="text">
                <option value="text">Text</option>
                <option value="integer">Integer</option>
                <option value="decimal">Decimal</option>
                <option value="date">Date</option>
                <option value="week">Week</option>
                <option value="dropdown">Dropdown</option>
                <option value="boolean">Boolean</option>
            </select>`

class TypeSettingsScreen extends HTMLElement {
    connectedCallback() {


        this.innerHTML = `
        <h1>Type Settings</h1>
        <h2>Types</h2>
        <form id='add_type'>
            <label>Name:</label>
            <input type="text" name="name">
            <label>Description</label>
            <input type="text" name="description">
            <label>Instructions</label>
            <button type="submit">Add Item Type</button>
        </form>
        <div id='types_container'></div>
        
        <h2>Attribute Kinds</h2>
        <form id='add_attribute_kind'>
            <label>Name:</label>
            <input type="text" name="name">
            <label>Description</label>
            <input type="text" name="description">
            <label>Kind</label>
            ${selectHTML}
            <button type="submit">Add Attribute Kind</button>
        </form>
        <div id='attribute_kinds_container'></div>
        `;

        (this.querySelector('#add_type') as HTMLFormElement)!.addEventListener('submit', (e: SubmitEvent) => {
            e.preventDefault();
            const data = new FormData(this.querySelector('#add_type')! as HTMLFormElement);
            const item_type = {
                name: data.get('name')! as string,
                description: data.get('description')! as string,
                // instructions: {
                    // required_fields: []
                // }
            }
            API.item.Types.add(item_type)
                .then(() => {
                    this.refresh();
                })
            // let settings = get_settings()
            // settings['item_types'].push(item_type);
            // AuthAPI.sync_settings();
            // this.refresh();
        });

        (this.querySelector('#add_attribute_kind') as HTMLFormElement)!.addEventListener('submit', (e: SubmitEvent) => {
            e.preventDefault();
            const data = new FormData(this.querySelector('#add_attribute_kind')! as HTMLFormElement);
            API.item.AttributeKinds.add(
                data.get('name') as string,
                data.get('description') as string,
                data.get('base_type') as string
            )
                .then(() => {
                    this.refresh();
                });


            // let settings = get_settings();
            // settings['attribute_metas'].push(attribute_kind);
            // AuthAPI.sync_settings();
            // this.refresh();
        });

        this.refresh();
    }

    async refresh() {
        let types_container = this.querySelector("#types_container")!;
        try {
            let item_types = await API.item.Types.get_all();
            types_container.innerHTML = "";
            item_types.forEach(item_type => {
                let view = document.createElement('div');
                view.classList.add('item_type')
                view.innerHTML =
                    `<input name="name" value="${item_type['name']}">
                    <input name="description" value="${item_type['description']}">
                    <button class="delete_button"><img style="width: 1em" src="${DeleteIcon}"></button>`
                let name_input = view.querySelector('[name="name"]')! as HTMLInputElement;
                let description_input = view.querySelector('[name="description"]')! as HTMLInputElement;
                let delete_button = view.querySelector('.delete_button')! as HTMLButtonElement;
                name_input.addEventListener('change', () => {
                    item_type.name = name_input.value;
                    API.item.Types.update(item_type.type_id!, {name: item_type.name})
                    this.refresh();
                })
                description_input.addEventListener('change', () => {
                    item_type.description = description_input.value;
                    API.item.Types.update(item_type.type_id!, {description: item_type.description})
                    this.refresh();
                })
                delete_button.addEventListener('click', () => {
                    API.item.remove(item_type.type_id!)
                        .then(() => {
                            view.remove();
                        })
                })
                types_container.appendChild(view);
            })
        } catch (e) {
            console.error(e)
        }

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
    

    disconnectedCallback() {

    }
}

customElements.define('type-settings-screen', TypeSettingsScreen);

export default TypeSettingsScreen;