import API from "../../api/api";
import { get_attribute_kinds, get_kind_for_key, type AttributeKind } from "../../api/attribute_kind";
import ItemAPI, { type Item } from "../../api/item";
import AddIcon from "../../assets/icon/add.svg";
import SettingsIcon from "../../assets/icon/settings.svg";
import { events } from "../../core/events";
import BaseElement from "../common/base_element";
import ChipsInput from "../common/chips_input";
import DeleteIcon from "../../assets/icon/close.svg";
import AttributeItemView from "./attribute_item_view";
import IconButton from "../common/icon_button";
import { EditIcon } from "../../assets/asset_map";


class AttributesView extends BaseElement<Item> {
    async render() {
        let item = this.data!;

        this.innerHTML = `
        <datalist id="attribute_kind_suggestions"></datalist>
        <div name="attribute_container"></div>
        <div name="add_container"></div>
        `
        // Datalist
        let suggestions = this.querySelector('#attribute_kind_suggestions')! as HTMLDataListElement;
        for (const [key, value] of Object.entries(item.attributes!)) {
            suggestions.innerHTML += `<option value=${key}></option>`
        }

        this.refresh_add_form();
        this.refresh_attribute_views();
    }
    
    private refresh_add_form() {
        let item = this.data!;

        let container = this.querySelector('[name="add_container"]')! as HTMLDivElement;
        container.style.display = "grid";
        container.style.gridTemplateColumns = "1fr 1.5em";

        let attr_view: AttributeItemView = new AttributeItemView().init({
            key: "",
            value: "",
        }) as AttributeItemView;
        let submit = new IconButton(AddIcon, 'Add New Attribute', async () => {
            let key = attr_view.key;
            let value = attr_view.value;
            if (key in item.attributes!) {
                return;
            }
            await API.item.Attributes.set_value(item.item_id, key, value);
            this.data!.attributes![key] = value;
            // Clear field.
            attr_view.init({
                key: "",
                value: ""
            })
            this.refresh_attribute_views();
        });

        attr_view.addEventListener('attr-add', (e) => {
            let attr = e.detail.attr;
            if (attr.key in item.attributes!) {
                return;
            }
            this.data!.attributes![attr.key] = attr.value;
            this.refresh_attribute_views();
        })
        container.appendChild(attr_view);
        container.appendChild(submit)
    }

    private async refresh_attribute_views() {
        let item = this.data!;

        let container = this.querySelector('[name="attribute_container"]')! as HTMLDivElement;
        container.style.display = "grid";
        container.style.gridTemplateColumns = "1fr 1.5em";

        container.innerHTML = "";

        for (const [key, value] of Object.entries(item.attributes!)) {
            let attr_item_view = new AttributeItemView().init({
                key,
                value,
            })
            attr_item_view.addEventListener('attr-rename', async (e) => {
                await API.item.Attributes.rename(item.item_id!, 
                    e.detail.old_key,
                    e.detail.new_key, 
                );
            })
            attr_item_view.addEventListener('attr-value-change', async (e) => {
                await API.item.Attributes.set_value(item.item_id!,
                    e.detail.key,
                    e.detail.new_value,
                );
            })
            let submit = new IconButton(DeleteIcon, 'Delete Attribute', async () => {
                await API.item.Attributes.remove(item.item_id!,
                    key
                );
                attr_item_view.remove();
                submit.remove();
            });

            container.appendChild(attr_item_view);
            container.appendChild(submit);
        }
    }
}



customElements.define('attributes-view', AttributesView)

export default AttributesView;