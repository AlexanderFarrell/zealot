import ItemAPI from "../api/item";
import { events } from "../core/events";
import DeleteIcon from "../assets/icon/delete.svg";
import { item_attribute_view } from "../components/sidebars/item_attributes_view";
import AttributesView from "../components/attributes_view";

class ItemScreen extends HTMLElement {
    public title: string = "";
    public content: string = "";
    public item: any;

    private switch_item = (data: any) => {
        this.title = data.title;
        this.innerHTML = ""
        this.render();
    };

    connectedCallback() {
        this.title = 'Home';
        events.on('switch_item', this.switch_item);
        this.render();
    }

    disconnectedCallback() {
        events.off('switch_item', this.switch_item);
        item_attribute_view?.clear();
    }

    async render() {
        this.innerHTML = ""
        try {
            this.item = await ItemAPI.get_by_title(this.title) as any;
            this.title = this.item.title;
            this.content = this.item.content;
        }
        catch (e) {
            console.error(e)
            this.innerHTML = `<div id='error'>Error getting item: ${this.title}</div>`
            return;
        }

        item_attribute_view?.switch_item(this.item);
        let title = document.createElement('div')
        title.id = "item_title"
        title.contentEditable = 'true';
        title.innerText = this.title;
        title.addEventListener('input', () => {
            // Update title
            this.title = title.textContent;
        });
        title.addEventListener('blur', () => {
            ItemAPI.update(this.item['item_id'], {'title': this.title});
            this.item['title'] = this.title;
        });

        let content = document.createElement('div');
        content.id = "item_content"
        content.contentEditable = 'true';
        content.innerText = this.item['content'];
        content.addEventListener('input', () => {
            this.content = content.textContent;
        })
        content.addEventListener('blur', () => {
            ItemAPI.update(this.item['item_id'], {'content': this.content});
            this.item['content'] = this.content;
        })

        let topContainer = document.createElement('div');
        topContainer.id = "top_container"
        topContainer.appendChild(title);


        let deleteButton = document.createElement('button');
        deleteButton.innerHTML = `<img src="${DeleteIcon}" alt="Delete Icon" title="Delete Item">`
        deleteButton.addEventListener('click', () => {
            if (confirm("Are you sure you want to delete this?")) {
                ItemAPI.remove(this.item['item_id'])
                    .then(() => {
                        switch_item_to("Home");
                    })
            }
        })
        topContainer.appendChild(deleteButton)

        this.appendChild(topContainer);
        this.appendChild(new AttributesView().setup(this.item['item_id'], this.item['attributes']));
        this.appendChild(content);
    }
}

export function switch_item_to(title: string) {
    events.emit('switch_item', {title: title})
}

customElements.define('item-screen', ItemScreen)

export default ItemScreen;