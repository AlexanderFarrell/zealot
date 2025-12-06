
import API from "../../api/api";
import DeleteIcon from "../../assets/icon/delete.svg";
import AddIcon from "../../assets/icon/add.svg"
import ChipsInput from "../../components/common/chips_input";


class TypeSettingsScreen extends HTMLElement {
    connectedCallback() {


        this.innerHTML = `
        <h1>Type Settings</h1>
        <form id='add_type' class="box" 
            style="display: grid; grid-template-columns: 1fr 1fr 2fr auto; margin-bottom: 10px;">
            <label>Name:</label>
            <label>Description</label>
            <label>Attributes</label>
            <div>&nbsp;</div>
            <input type="text" name="name">
            <input type="text" name="description">
            <chips-input></chips-input>
            <button type="submit"><img src="${AddIcon}" style="width: 1em"></button>
        </form>
        <div id='types_container'></div>
        `;

        (this.querySelector('#add_type') as HTMLFormElement)!.addEventListener('submit', (e: SubmitEvent) => {
            e.preventDefault();
            const data = new FormData(this.querySelector('#add_type')! as HTMLFormElement);
            const item_type = {
                name: data.get('name')! as string,
                description: data.get('description')! as string,
            }
            API.item.Types.add(item_type)
                .then(() => {
                    this.refresh();
                })
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
                    <chips-input></chips-input>
                    <button class="delete_button"><img style="width: 1em" src="${DeleteIcon}"></button>`
                let name_input = view.querySelector('[name="name"]')! as HTMLInputElement;
                let description_input = view.querySelector('[name="description"]')! as HTMLInputElement;
                let attributes_input = view.querySelector('chips-input')! as ChipsInput;
                let delete_button = view.querySelector('.delete_button')! as HTMLButtonElement;

                attributes_input.set_value(item_type.required_attribute_keys || [])

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
                attributes_input.on_add((items: string[]) => {
                    item_type.required_attribute_keys!.push(...items);
                    API.item.Types.assign(items, item_type.name);
                })
                attributes_input.on_remove((items: string[]) => {
                    item_type.required_attribute_keys = item_type.required_attribute_keys!.filter((i => {
                        return !(i in items)
                    }))
                    API.item.Types.unassign(items, item_type.name)
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

    }
    

    disconnectedCallback() {

    }
}

customElements.define('type-settings-screen', TypeSettingsScreen);

export default TypeSettingsScreen;