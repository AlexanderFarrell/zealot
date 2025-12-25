import ItemAPI, { type Item } from "../api/item";
import { events } from "../core/events";
import DeleteIcon from "../assets/icon/delete.svg";
import { item_attribute_view } from "../components/sidebars/item_attributes_view";
import AttributesView from "../components/item/attributes_view";
import { router } from "../core/router";
import API from "../api/api";
import type ChipsInput from "../components/common/chips_input";

import BaseElement from "../components/common/base_element";
import PlanView from "../components/item_view";
import type AddItemScoped from "../components/add_item_scope";
import ContentView from "../components/item/content_view";
import ItemListView from "../components/item_list_view";

class ItemScreen extends BaseElement<Item> {
    private last_loaded_title: string | null = null;


    async render() {
        if (this.data == null) {
            this.render_empty_screen()
            return;
        }

        this.innerHTML = `
        <div class="title" name="title_container">
            <h1 name="title" contenteditable="true"></h1>
            <button name="delete_button" title="Delete Item"><img src="${DeleteIcon}" alt="Delete Button Icon"></button>
        </div>
        <div name="item_types" class="attribute"></div>
        <attributes-view></attributes-view>
        <content-view></content-view>
        <item-list-view style="padding-bottom: 1em;"></item-list-view>
        <!--<div id="children_container" style="padding-top: 4em"></div>
                    <div name="children">
                <div>Children</div>
                <add-item-scoped></add-item-scoped>
                <div name="children_container"></div>
            </div>-->
        `

        this.setup_title_view();
        this.setup_types_view();
        this.setup_attributes_view();
        this.setup_content_view();
        this.render_children();        
    }

    disconnectedCallback() {
        // let right_sidebar = document.querySelector('#right-side-bar')!;
        // right_sidebar.innerHTML = "";

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

        let on: (i: string) => void = (i) => {
            router.navigate(`/types/${i}`)
        }
        // @ts-ignore
        input.OnClickItem = on;
    }    
    
    
    async setup_content_view() {
        let view = this.querySelector('content-view')! as ContentView;
        view.init(this.data!);
    }    

    async render_children() {
        let item = this.data!;
        let children_view = this.querySelector('item-list-view')! as ItemListView;

        let children = await API.item.children(item.title);

        children_view
            .enable_add_item(
                {Parent: [this.data!.title]},
                async () => {
                    children_view.data = await API.item.children(item.title);
                }
            )
            .init(children);
        // let right_sidebar = document.querySelector('#right-side-bar')!;
        // right_sidebar.innerHTML = `
        //     <div name="children">
        //         <div>Children</div>
        //         <add-item-scoped></add-item-scoped>
        //         <div name="children_container"></div>
        //     </div>
        // `

        // let container = right_sidebar.querySelector('[name="children_container"]')!;
        // let container = this.querySelector('[name="children_container"]')!;


        // container.innerHTML = "";

        // // Get children
        // let children = await API.item.children(item.title);
        // children.forEach(child => {
        //     let view = new PlanView().init(child);
        //     container.appendChild(view)
        // });

        // // let add_child = right_sidebar.querySelector('add-item-scoped')! as AddItemScoped;
        // let add_child = this.querySelector('add-item-scoped')! as AddItemScoped;
        // add_child.init({
        //     Parent: [this.data!.title]
        // })
        // add_child.listen_on_submit(() => {this.render_children()})
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