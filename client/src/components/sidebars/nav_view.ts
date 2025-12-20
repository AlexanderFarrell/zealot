import API from "../../api/api";
import type { Item } from "../../api/item";
import { router } from "../../core/router";
import BaseElement, { BaseElementEmpty } from "../common/base_element";


class NavViewItem extends BaseElement<Item> {
    render() {
        this.innerHTML = `
        <div style="display: grid; grid-template-columns: auto 1fr;">
            <div name="expand_button"> > </div>
            <div name="title"></div>
        </div>
        <div name="children" style="padding-left: 1em"></div>
        `
        let expanded = false;

        let expand_button = this.querySelector('[name="expand_button"]')!;
        let title = this.querySelector('[name="title"]')! as HTMLDivElement;
        let child_view = this.querySelector('[name="chidren"]')!;

        title.innerText = this.data!.title;
        title.addEventListener('click', () => {
            router.navigate(`/item/${this.data!.title}`)
        })

        expand_button.addEventListener('click', async () => {
            expanded = !expanded;
            if (expanded) {
                let children = await API.item.children(this.data!.title);
                children.forEach(c => {
                    child_view.appendChild(new NavViewItem().init(c));
                })
            } else {
                child_view.innerHTML = "";
            }
        })
    }
}


class NavView extends BaseElementEmpty {
    async render() {
        this.innerHTML = "";

        let items = await API.item.root_items();
        items.forEach(i => {
            this.appendChild(new NavViewItem().init(i))
        })
        if (items.length == 0) {
            this.innerText = "Set Root to true on items to have them appear here."
        }
    }
}

customElements.define('nav-view', NavView)

export default NavView;