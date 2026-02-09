import API from "../../api/api";
import DeleteIcon from "../../assets/icon/delete.svg";
import { events } from "../../shared/events";
import AddIcon from "../../assets/icon/add.svg"
import BaseElement, {BaseElementEmpty, GenerateSelectHTML} from "../../shared/base_element";
import { AttributeKindsBaseTypes, type AttributeKind } from "../../api/attribute_kind";
import { Title } from "../../shared/string_helper";


class AttributeSettingsScreen extends BaseElementEmpty {
    async render() {
        this.innerHTML = `
        <h1>Attribute Kinds</h1>
        <form id='add_attribute_kind' class="box" style="display: grid; grid-template-columns: 1fr 2fr auto auto; margin-bottom: 10px">
            <label>Name:</label>
            <label>Description</label>
            <label>Kind</label>
            <div>&nbsp;</div>
            <input type="text" name="key">
            <input type="text" name="description">
            ${select_html_make()}
            <button type="submit"><img src="${AddIcon}" style="width: 1em;"></button>
        </form>
        <h2>System Kinds</h2>
        <div>These are built in and cannot be edited</div>
        <div id='system_kinds'></div>
        <h2>User Kinds</h2>
        <div id='user_kinds'></div>
        `;

        let attribute_kind_add_form = this.querySelector('#add_attribute_kind')! as HTMLFormElement;

        attribute_kind_add_form.addEventListener('submit', async (e: SubmitEvent) => {
            e.preventDefault();
            const data = new FormData(this.querySelector('#add_attribute_kind')! as HTMLFormElement);
            await API.item.AttributeKinds.add({
                key: data.get('key') as string,
                description: data.get('description') as string,
                base_type: data.get('base_type') as string,
                config: defaultConfigForType(data.get('base_type') as string)
            })
            this.render_kinds()
        });
        this.render_kinds();
    }

    async render_kinds() {
        let sys_kinds_view = this.querySelector('#system_kinds')! as HTMLDivElement;
        let user_kinds_view = this.querySelector('#user_kinds')! as HTMLDivElement;

        try {
            let kinds = await API.item.AttributeKinds.get_all();
            let sys_kinds = kinds.filter(k => k.is_system!)
            let user_kinds = kinds.filter(k => !k.is_system!)
            
            sys_kinds_view.innerHTML = "";
            sys_kinds.forEach((kind: any) => {
                sys_kinds_view.appendChild(new AttributeKindSettings().init(kind));
            })

            user_kinds_view.innerHTML = "";
            user_kinds.forEach((kind: any) => {
                user_kinds_view.appendChild(new AttributeKindSettings().init(kind));
            })
            if (user_kinds.length == 0) {
                user_kinds_view.innerHTML = "Attribute kinds you add will appear here."
            }
        }
        catch (e) {
            console.error(e)
        }
    }
}

class AttributeKindSettings extends BaseElement<AttributeKind> {
    render() {
        let kind = this.data!;

        this.classList.add('attribute_kind')
        this.innerHTML = 
        `<input name="key" value="${kind['key']}">
        <input name="description" value="${kind['description']}">
        ${select_html_make()}
        <button class="delete_button"><img style="width: 1em" src="${DeleteIcon}"></button>
        `;
        let key_input = this.querySelector('[name="key"]')! as HTMLInputElement;
        let description_input = this.querySelector('[name="description"]')! as HTMLInputElement;
        let base_type_input = this.querySelector('[name="base_type"]')! as HTMLSelectElement;
        let delete_button = this.querySelector('.delete_button')! as HTMLButtonElement;

        base_type_input.value = kind.base_type;
        if (kind.is_system) {
            key_input.disabled = true;
            description_input.disabled = true;
            base_type_input.disabled = true;
            delete_button.disabled = true;
        } else {
            key_input.addEventListener('change', () => {
                kind.key = (this.querySelector('[name="key"]')! as HTMLInputElement).value;
                API.item.AttributeKinds.update(kind.kind_id!, {key: kind.key})
                events.emit('refresh_attributes')
            })
            description_input.addEventListener('change', () => {
                kind.description = (this.querySelector('[name="description"]')! as 
                HTMLInputElement).value;
                API.item.AttributeKinds.update(kind.kind_id!, {description: kind.description})
                events.emit('refresh_attributes')
            })
            base_type_input.addEventListener('change', () => {
                kind.base_type = base_type_input.value;
                API.item.AttributeKinds.update(kind.kind_id!, {base_type: kind.base_type})
                events.emit('refresh_attributes')
            })
            delete_button.addEventListener('click', async () => {
                await API.item.AttributeKinds.remove(kind.kind_id!)
                events.emit('refresh_attributes')
                this.remove()
            })
        }
    }
}

let select_html_make = () => {
    return GenerateSelectHTML('base_type', 'text',
        AttributeKindsBaseTypes,
        AttributeKindsBaseTypes.map(a => Title(a))
    )
}

function defaultConfigForType(base_type: string): object {
    switch (base_type) {
        case "integer":
            return {min: null, max: null};
        case "decimal":
            return {min: null, max: null};
        case "text":
            return {min_len: null, max_len: null, pattern: null};
        case "date":
            return {};
        case "week":
            return {};
        case "dropdown":
            return {values: ["todo", "working", "complete"]};
        case "boolean":
            return {};
        case "list":
            return {list_type: "text"};
        default:
            return {}
    }
}

// function createConfigInputs(base_type: string): HTMLElement {

// }

customElements.define('attr-settings-screen', AttributeSettingsScreen)
customElements.define('attr-kind', AttributeKindSettings)
export default AttributeSettingsScreen;