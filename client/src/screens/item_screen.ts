import ItemAPI, { type Item } from "../api/item";
import { events } from "../core/events";
import DeleteIcon from "../assets/icon/delete.svg";
import { item_attribute_view } from "../components/sidebars/item_attributes_view";
import AttributesView from "../components/attributes_view";
import { router } from "../core/router";
import API from "../api/api";


class ItemScreen extends HTMLElement {
    private _item: Item | null = null;

    // Views
    private title_view!: HTMLHeadingElement;
    private attribute_view!: AttributesView;
    private content_view!: HTMLElement;

    public set item(value: Item) {
        this._item = value;
        this.render()
    }

    public get item(): Item | null {
        return this._item;
    }

    public async LoadItem(title: string) {
        try {
            this.item = await ItemAPI.get_by_title(title);
        } catch (e) {
            this.innerHTML = "<div class='error'>Error loading item</div>";
        }
    }

    connectedCallback() {
        this.innerHTML = "Loading..."
    }

    disconnectedCallback() {
    }

    async render() {
        this.innerHTML = `
        <div name="title_container">
            <h1 name="title" contenteditable="true"></h1>
            <button name="delete_button" title="Delete Item"><img src="${DeleteIcon}" alt="Delete Button Icon"></button>
        </div>
        <attributes-view></attributes-view>
        <div name="content" contenteditable="true"></div>
        `
        this.title_view = this.querySelector('[name="title"]')!;
        this.attribute_view = this.querySelector('attributes-view')!;
        this.content_view = this.querySelector('[name="content"]')!;

        // Title
        this.title_view.contentEditable = 'true';
        this.title_view.innerText = this.item!.title;
        this.title_view.addEventListener('input', () => {
            this.item!.title = this.title_view.textContent;
        })
        this.title_view.addEventListener('blur', () => {
            ItemAPI.update(this.item!.item_id, {title: this.item!.title})
        })

        // Delete Button
        let delete_button = this.querySelector('[name="delete_button"]')!;
        delete_button.addEventListener('click', async () => {
            if (confirm("Are you sure you want to delete this?")) {
                await ItemAPI.remove(this.item!.item_id)
                router.navigate('/')
            }
        })

        // Attributes View
        this.attribute_view.setup(this.item!.item_id, this.item!.attributes!);

        // Content
        this.content_view.innerText = this.item!.content;
        this.content_view.addEventListener('input', () => {
            this.item!.content = this.content_view.textContent;
        })
        this.content_view.addEventListener('blur', () => {
            ItemAPI.update(this.item!.item_id, {content: this.item!.content});
        })
    }
}

customElements.define('item-screen', ItemScreen)

export default ItemScreen;