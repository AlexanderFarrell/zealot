import type { Item } from "../../api/item";

class ItemAttributesView extends HTMLElement {
    private LinkedView: HTMLElement | null = null;
    private MetaView: HTMLElement | null = null;
    private Item: Item | null = null;

    constructor() {
        super();
    }

    connectedCallback() {
        if (item_attribute_view != null) {
            console.error("Multiple Item Attribute Views")
        }
        item_attribute_view = this;

        this.innerHTML = "";

        this.LinkedView = document.createElement('div');
        this.appendChild(this.LinkedView);
        
        this.MetaView = document.createElement('div');
        this.appendChild(this.MetaView)
    }

    disconnectedCallback() {
        item_attribute_view = null;
    }

    clear() {

    }

    switch_item(item: Item) {
        this.Item = item;
        // Attributes
        this.update_linked();
        this.update_meta();
    }

    update_meta() {
        this.MetaView!.innerHTML = "<h2>Summary</h2>"
        this.MetaView!.innerHTML += "<p>Currently not supported</p>"
    }


    update_linked() {
        this.LinkedView!.innerHTML = "<h2>Linked</h2>";
        this.LinkedView!.innerHTML += "<p>Currently not supported</p>"
    }


}

customElements.define('item-attributes-view', ItemAttributesView)
export let item_attribute_view: ItemAttributesView | null = null;

export default ItemAttributesView;