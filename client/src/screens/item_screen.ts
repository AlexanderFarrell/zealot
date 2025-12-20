import ItemAPI, { type Item } from "../api/item";
import { events } from "../core/events";
import DeleteIcon from "../assets/icon/delete.svg";
import { item_attribute_view } from "../components/sidebars/item_attributes_view";
import AttributesView from "../components/item/attributes_view";
import { router } from "../core/router";
import API from "../api/api";
import type ChipsInput from "../components/common/chips_input";
import EditorJS, { type OutputData } from "@editorjs/editorjs";

import Header from "@editorjs/header";
import List from "@editorjs/list";
import Quote from "@editorjs/quote";
import Code from "@editorjs/code";
import Delimiter from "@editorjs/delimiter";
import BaseElement from "../components/common/base_element";
import PlanView from "../components/plan_view";
import type AddItemScoped from "../components/add_item_scope";

class ItemScreen extends BaseElement<Item> {
    private last_loaded_title: string | null = null;


    async render() {
        if (this.data == null) {
            this.render_empty_screen()
            return;
        }

        this.innerHTML = `
        <div name="title_container">
            <h1 name="title" contenteditable="true"></h1>
            <button name="delete_button" title="Delete Item"><img src="${DeleteIcon}" alt="Delete Button Icon"></button>
        </div>
        <div name="item_types" class="attribute"></div>
        <attributes-view></attributes-view>
        <div id="content_holder"></div>
        <div name="children">
            <add-item-scoped></add-item-scoped>
            <div name="children_container"></div>
        </div>
        `

        this.setup_title_view();
        this.setup_types_view();
        this.setup_attributes_view();
        this.setup_content_view();
        this.render_children();        
        
        let add_child = this.querySelector('add-item-scoped')! as AddItemScoped;
        add_child.init({
            Parent: [this.data!.title]
        })
        add_child.listen_on_submit(() => {this.render_children()})
    }


    async setup_title_view() {
        let title = this.querySelector('[name="title"]')! as HTMLHeadingElement;        
        title.innerText = this.data!.title;
        title.addEventListener('input', () => {
            this.data!.title = title.textContent;
        })
        title.addEventListener('blur', () => {
            ItemAPI.update(this.data!.item_id, {title: this.data!.title})
        })        
        let delete_button = this.querySelector('[name="delete_button"]')!;
        delete_button.addEventListener('click', async () => {
            if (confirm("Are you sure you want to delete this?")) {
                await ItemAPI.remove(this.data!.item_id)
                router.navigate('/')
            }
        })
    }


    async setup_attributes_view() {
        let attribute_view = this.querySelector('attributes-view')! as AttributesView;
        attribute_view.init(this.data!);
    }


    async setup_types_view() {
        let types_view = this.querySelector('[name="item_types"]')!;
        types_view.innerHTML = `
        <input disabled value="Types">
        <chips-input></chips-input>
        <div style="visibility: hidden;"><img src="${DeleteIcon}"</div>
        `
        let input = types_view.querySelector('chips-input')! as ChipsInput;

        input.set_value(this.data!.types!.map(it => it.name));

        input.addEventListener('chips-add', (e) => {
            let items = e.detail.items;
            try {
                items.forEach((item) => {
                    API.item.assign_type(this.data!.item_id, item)
                })
            } catch (e) {
                console.error(e)
            }
        })

        input.addEventListener('chips-remove', (e) => {
            let items = e.detail.items;
            try {
                items.forEach(item => {
                    API.item.unassign_type(this.data!.item_id, item)
                })
            } catch (e) {
                console.error(e)
            }
        })
    }    
    
    
    async setup_content_view() {
        let data: OutputData | undefined = undefined;
        try {
            data = JSON.parse(this.data!.content) as OutputData;
        } catch (e) {
            console.error(e)
        }

        // Content
        let content_view = new EditorJS({
            holder: "content_holder",
            // readOnly: !!opts.readOnly,
            placeholder: "Write content here...",
            data: data,
            // data: this.item!.content,
            // autofocus: !opts.readOnly,
            inlineToolbar: ["bold", "italic", "link"],

            tools: {
            paragraph: {
                // paragraph is built-in, no import needed

            },
            header: {
                // @ts-ignore
                class: Header,
                inlineToolbar: true,
                config: {
                levels: [1, 2, 3, 4],
                defaultLevel: 1,
                },
            },
            list: {
                class: List,
                inlineToolbar: true,
                config: {
                defaultStyle: "unordered",
                },
            },
            quote: {
                class: Quote,
                inlineToolbar: true,
                config: {
                quotePlaceholder: "Quote",
                captionPlaceholder: "Source",
                },
            },
            code: {
                class: Code,
            },
            delimiter: {
                class: Delimiter,
            },
            },

            onChange: async () => {
                let output = await content_view.save();
                ItemAPI.update(this.data!.item_id, {content: JSON.stringify(output)})
            },
        });
    }    

    async render_children() {
        let item = this.data!;
        let container = this.querySelector('[name="children_container"]')!;


        container.innerHTML = "";

        // Get children
        let children = await API.item.children(item.title);
        children.forEach(child => {
            let view = new PlanView();
            view.item = child;
            container.appendChild(view)
        });
    }
    
    async render_empty_screen() {
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
            this.data = await API.item.get_by_title(this.last_loaded_title!);
        })
    }
    
    public async LoadItem(title: string) {
        this.last_loaded_title = title;
        try {
            this.data = await ItemAPI.get_by_title(title);
        } catch (e) {
            this.innerHTML = "<div class='error'>Error loading item</div>";
        }
    }
}

customElements.define('item-screen', ItemScreen)

export default ItemScreen;