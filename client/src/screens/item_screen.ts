import mitt from "mitt";
import ItemAPI from "../api/item";
import { events } from "../core/events";

class ItemScreen extends HTMLElement {
    public title: string = "";
    public item: any;
    private switch_item = (data: any) => {
        this.title = data.title;
        this.render();
    };

    connectedCallback() {
        this.title = 'Home';
        events.on('switch_item', this.switch_item);
        this.render();
    }

    disconnectedCallback() {
        events.off('switch_item', this.switch_item);
    }

    async render() {
        try {
            this.item = await ItemAPI.get_by_title(this.title) as any;
            this.title = this.item.title;
            this.innerHTML = `<div id="item_title">${this.item.title}</div><div id="item_content">${this.item.content}</div>`
        }
        catch (e) {
            console.error(e)
            this.innerHTML = `<div id='error'>Error getting item: ${this.title}</div>`
        }
    }
}

export function switch_item_to(title: string) {
    document.querySelector('content-')!.innerHTML = "<item-screen></item-screen>";
    events.emit('switch_item', {title: title})
}

customElements.define('item-screen', ItemScreen)

export default ItemScreen;