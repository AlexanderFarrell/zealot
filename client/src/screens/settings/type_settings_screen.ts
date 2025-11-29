import AuthAPI from "../../api/auth";
import { get_settings } from "../../core/settings";

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
            <select value="text">
                <option value="text">Text</option>
                <option value="number">Number</option>
            </select>
            <button type="submit">Add Attribute Kind</button>
        </form>
        <div id='attribute_kinds_container'></div>
        `;

        (this.querySelector('#add_type') as HTMLFormElement)!.addEventListener('submit', (e: SubmitEvent) => {
            e.preventDefault();
            const data = new FormData(this.querySelector('#add_type')! as HTMLFormElement);
            const item_type = {
                name: data.get('name')!,
                description: data.get('description')!,
                instructions: {
                    required_fields: []
                }
            }
            let settings = get_settings()
            settings['item_types'].push(item_type);
            AuthAPI.sync_settings();
            this.refresh();
        });

        (this.querySelector('#add_attribute_kind') as HTMLFormElement)!.addEventListener('submit', (e: SubmitEvent) => {
            e.preventDefault();
            const data = new FormData(this.querySelector('#add_attribute_kind')! as HTMLFormElement);
            const attribute_kind = {
                name: data.get('name'),
                description: data.get('description'),
                kind: data.get('kind')
            };
            let settings = get_settings();
            settings['attribute_metas'].push(attribute_kind);
            AuthAPI.sync_settings();
            this.refresh();
        });

        this.refresh();
    }

    refresh() {
        let settings = get_settings();
        let types_container = this.querySelector("#types_container")!;
        types_container.innerHTML = "";
        settings['item_types'].forEach((item_type: any) => {
            let view = document.createElement('div');
            view.classList.add('item_type');
            view.innerHTML =
                `<input name="name" value="${item_type['name']}">
                <input name="description" value="${item_type['description']}">`
            view.querySelector('[name="name"]')!.addEventListener('change', () => {
                item_type.name = (view.querySelector('[name="name"]')! as HTMLInputElement).value;
                AuthAPI.sync_settings();
                this.refresh();
            })
            view.querySelector('[name="description"]')!.addEventListener('change', () => {
                item_type.description = (view.querySelector('[name="description"]')! as 
                HTMLInputElement).value;
                AuthAPI.sync_settings();
                this.refresh();
            })
            types_container.appendChild(view);
        });

        let attribute_kinds_container = this.querySelector('#attribute_kinds_container')!;
        attribute_kinds_container.innerHTML = "";
        settings['attribute_metas'].forEach((attribute_kind: any) => {
            let view = document.createElement('div');
            view.classList.add('attribute_kind')
            view.innerHTML = 
            `<input name="name" value="${attribute_kind['name']}">
            <input name="description" value="${attribute_kind['description']}">
            `            
            view.querySelector('[name="name"]')!.addEventListener('change', () => {
                attribute_kind.name = (view.querySelector('[name="name"]')! as HTMLInputElement).value;
                AuthAPI.sync_settings();
                this.refresh();
            })
            view.querySelector('[name="description"]')!.addEventListener('change', () => {
                attribute_kind.description = (view.querySelector('[name="description"]')! as 
                HTMLInputElement).value;
                AuthAPI.sync_settings();
                this.refresh();
            })
            attribute_kinds_container.appendChild(view)
        })
    }
    

    disconnectedCallback() {

    }
}

customElements.define('type-settings-screen', TypeSettingsScreen);

export default TypeSettingsScreen;