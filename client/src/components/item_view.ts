import type { Item } from "../api/item";
import { router } from "../core/router";
import BaseElement from "./common/base_element";


class ItemView extends BaseElement<Item> {
    async render() {
        let item = this.data!;
        this.innerHTML = `
        <div name="icon"></div>
        <div name="title"></div>
        <div name="status"></div>
        <div name="priority"></div>
        `

        let icon_view = this.querySelector('[name="icon"]')! as HTMLElement;
        let title_view = this.querySelector('[name="title"]')! as HTMLElement;
        let status_view = this.querySelector('[name="status"]')! as HTMLElement;
        let priority_view = this.querySelector('[name="priority"]')! as HTMLElement;

        title_view.innerText = item.title;

        let set_attr = (view: HTMLElement, attribute: string) => {
            if (item.attributes![attribute] != null) {
                view.innerText = item.attributes![attribute];
            } else {
                view.innerHTML = "&nbsp;"
            }
        }

        set_attr(icon_view, "Icon")
        set_attr(status_view, 'Status')
        set_attr(priority_view, "Priority")

        this.addEventListener('click', () => {
            router.navigate(`/item_id/${item.item_id}`)
            // router.navigate(`/item/${encodeURIComponent(item.title)}`);
        })
    }
}

customElements.define('item-view', ItemView)

export default ItemView;