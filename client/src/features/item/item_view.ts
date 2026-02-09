import API from "../../api/api";
import type { Item } from "../../api/item";
import DragUtil from "./drag_helper";
import { router } from "../router/router";
import BaseElement from "../../shared/base_element";
import type { AttributeValueView } from "./view/attribute_item_view";

class ItemView extends BaseElement<Item> {
    async render() {
        let item = this.data!;
        this.innerHTML = `
        <div name="title"></div>
        <attribute-value-view name="status"></attribute-value-view>
        <attribute-value-view name="priority"></attribute-value-view>
        `

        let title_view = this.querySelector('[name="title"]')! as HTMLElement;
        let status_view = this.querySelector('[name="status"]')! as AttributeValueView;
        let priority_view = this.querySelector('[name="priority"]')! as AttributeValueView;

        title_view.innerText = (item.attributes!['Icon'] || '🔵') + " " + item.title;

        // Status
        status_view.init({
            key: "Status",
            value: item.attributes!["Status"]
        })
        status_view.addEventListener('change', async () => {
            await API.item.Attributes.set_value(item.item_id, 'Status', status_view.value);
            this.dispatchEvent(new Event('change', {bubbles: true}));
        })

        priority_view.init({
            key: "Priority",
            value: item.attributes!['Priority']
        });
        priority_view.addEventListener('change', async () => {
            await API.item.Attributes.set_value(item.item_id, 'Priority', priority_view.value);
            this.dispatchEvent(new Event('change', {bubbles: true}));
        })

        DragUtil.setup_drag(this, item);
        DragUtil.setup_drop(this, {"Parent": [item.title]});

        title_view.addEventListener('click', () => {
            router.navigate(`/item_id/${item.item_id}`)
        })
    }
}

customElements.define('item-view', ItemView)

export default ItemView;