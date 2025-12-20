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


class AttributesView extends BaseElement<Item> {
    async render() {
        let item = this.data!;

        this.innerHTML = `
        <datalist id="attribute_kind_suggestions">
        
        </datalist>
        <div name="add_container"></div>
        <div name="attribute_container"></div>
        <!--<form class="attribute">
            <input type="text" name="key" list="attribute_kind_suggestions" required>
            <input type="text" name="value" required>
            <button type="submit"><img src="${AddIcon}" style="width: 1em"></button>
        </form>-->
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

        let add_container = this.querySelector('[name="add_container"]')! as HTMLDivElement;
        let add_attr_view = new AttributeItemView().init({
            item_id: item.item_id,
            key: "",
            value: "",
            is_new: true,
        })
        add_attr_view.addEventListener('attr-add', (e) => {
            let attr = e.detail.attr;
            if (attr.key in item.attributes!) {
                return;
            }
            this.data!.attributes![attr.key] = attr.value;
            this.refresh_attribute_views();
        })
        add_container.appendChild(add_attr_view);
    }

    private async refresh_attribute_views() {
        let item = this.data!;
        let container = this.querySelector('[name="attribute_container"]')!
        container.innerHTML = "";

        for (const [key, value] of Object.entries(item.attributes!)) {
            // this.add_key_value_input(key, value);
            container.appendChild(new AttributeItemView().init({
                item_id: item.item_id!,
                key,
                value,
                is_new: false
            }))
        }
    }
}



customElements.define('attributes-view', AttributesView)

export default AttributesView;