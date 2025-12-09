import ItemAPI, { type Item } from "../api/item";
import { events } from "../core/events";
import DeleteIcon from "../assets/icon/delete.svg";
import { item_attribute_view } from "../components/sidebars/item_attributes_view";
import AttributesView from "../components/attributes_view";
import { router } from "../core/router";
import API from "../api/api";
import type ChipsInput from "../components/common/chips_input";
import ContentView from "./item/content_view";


class ItemScreen extends HTMLElement {
    private _item: Item | null = null;

    // Views
    private title_view!: HTMLHeadingElement;
    private attribute_view!: AttributesView;
    private content_view!: ContentView;
    // private content_view!: HTMLTextAreaElement;
    private item_types_view!: HTMLElement;
    private last_loaded_title: string | null = null;

    public set item(value: Item) {
        this._item = value;
        this.render()
    }

    public get item(): Item | null {
        return this._item;
    }

    public async LoadItem(title: string) {
        this.last_loaded_title = title;
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
        if (this.item == null) {
            this.innerHTML = `That item doesn't exist.`
            if (this.last_loaded_title) {
                this.innerHTML += '<button>Create it?</button>'
            }
            this.querySelector('button')?.addEventListener('click', async () => {
                let item = {
                    title: this.last_loaded_title!,
                    content: '',
                    item_id: -1
                }
                await API.item.add(item)
                this.item = await API.item.get_by_title(this.last_loaded_title!);
            })

            return;
        }

        this.innerHTML = `
        <div name="title_container">
            <h1 name="title" contenteditable="true"></h1>
            <button name="delete_button" title="Delete Item"><img src="${DeleteIcon}" alt="Delete Button Icon"></button>
        </div>
        <div name="item_types" class="attribute"></div>
        <attributes-view></attributes-view>
        <content-view></content-view>
        `
        this.title_view = this.querySelector('[name="title"]')!;
        this.attribute_view = this.querySelector('attributes-view')!;
        this.content_view = this.querySelector('content-view')! as ContentView;
        this.item_types_view = this.querySelector('[name="item_types"]')!;

        this.content_view.init(this.item);

        this.render_item_types_view();

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
        this.attribute_view.item = this.item!;

        // Content
        // this.content_view.value = this.item!.content;
        // this.content_view.addEventListener('input', () => {
        //     this.item!.content = this.content_view.value;
        // })
        // this.content_view.addEventListener('blur', () => {
        //     ItemAPI.update(this.item!.item_id, {content: this.item!.content});
        // })
    }

    async render_item_types_view() {
        this.item_types_view.innerHTML = `
        <input disabled value="Types">
        <chips-input></chips-input>
        <div style="visibility: hidden;"><img src="${DeleteIcon}"</div>
        `
        let input = this.item_types_view.querySelector('chips-input')! as ChipsInput;

        input.set_value(this.item!.types!.map(it => it.name));

        input.on_add((items: string[]) => {
            try {
                items.forEach((item) => {
                    API.item.assign_type(this.item!.item_id, item)
                })
            } catch (e) {
                console.error(e)
            }
        })

        input.on_remove((items: string[]) => {
            try {
                items.forEach(item => {
                    API.item.unassign_type(this.item!.item_id, item)
                })
            } catch (e) {
                console.error(e)
            }
        })
    }
}

customElements.define('item-screen', ItemScreen)

export default ItemScreen;