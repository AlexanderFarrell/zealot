import ItemAPI, { type Item } from "../api/item";
import { events } from "../core/events";
import DeleteIcon from "../assets/icon/delete.svg";
import { item_attribute_view } from "../components/sidebars/item_attributes_view";
import AttributesView from "../components/item/attributes_view";
import { router } from "../core/router";
import API from "../api/api";
import type ChipsInput from "../components/common/chips_input";
import { DateTime } from "luxon";

import BaseElement from "../components/common/base_element";
import PlanView from "../components/item_view";
import type AddItemScoped from "../components/add_item_scope";
import ContentView from "../components/item/content_view";
import ItemListView from "../components/item_list_view";
import Popups from "../core/popups";
import runner from "../core/command_runner";
import ButtonGroup, { ButtonDef } from "../components/common/button_group";
import { CopyIcon, DownloadIcon, EditIcon, ItemsIcon, LinkIcon, ScienceIcon, UpIcon } from "../assets/asset_map";
import createZealotEditorView, { createZealotEditorState } from "../components/zealotscript_editor";
import { serializeZealotScript } from "../core/zealotscript/serializer";

let content_visible = true;
let related_visible = true;

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

        </div>
        <div name="item_types" class="attribute"></div>
        <attributes-view></attributes-view>
        <content-view></content-view>
        <item-list-view style="padding-bottom: 1em;"></item-list-view>
        <div class="item-comments comments-section">
            <div style="display: grid; grid-template-columns: 1fr auto">
                <h2>Comments</h2>
                <button type="button" name="item_comments_toggle">Add</button>
            </div>
            <form name="item_comments_form" class="comments-form" style="display: none;">
                <div class="comments-form-row">
                    <input type="time" name="item_comments_time">
                    <button type="submit">Log</button>
                </div>
                <div class="comments-form-editor" name="item_comments_editor"></div>
            </form>
            <div name="item_comments_list" style="display: grid; gap: 8px;"></div>
        </div>
        `
        this.prepend(new ButtonGroup().init([
            new ButtonDef(
                UpIcon,
                'To Parent',
                () => {
                    if (this.data!.attributes!['Parent'] != null) {
                        let first_parent = this.data!.attributes!['Parent'][0];
                        router.navigate(`/item/${first_parent}`)
                    } else {
                        router.navigate('/')
                    }
                }
            ),
            new ButtonDef(
                EditIcon,
                'Toggle Content Editor',
                () => {
                    // These could be global so that other items are with this view
                    content_visible = !content_visible;
                    let view = this.querySelector('content-view')! as ContentView;
                    view.style.display = (content_visible) ? 'block' : 'none';
                }
            ),
            new ButtonDef(
                ItemsIcon,
                'Toggle Related Items',
                () => {
                    related_visible = !related_visible;
                    let view = this.querySelector('content-view')! as ItemListView;
                    view.style.display = (related_visible) ? 'block' : 'none';
                }
            ),
            // new ButtonDef(
            //     ScienceIcon,
            //     'Toggle Analysis',
            //     () => {

            //     }
            // ),
            new ButtonDef(
                LinkIcon,
                'Copy Link',
                () => {
                    navigator.clipboard.writeText(window.location.href);
                }
            ),
            new ButtonDef(
                DownloadIcon,
                'Export',
                () => {

                }
            ),
            new ButtonDef(
                CopyIcon,
                'Copy as JSON',
                () => {
                    navigator.clipboard.writeText(JSON.stringify(this.data!));
                    // Make this a command so that practically any page could be copied as JSON
                }
            ),
            new ButtonDef(
                DeleteIcon,
                'Delete Item',
                async () => {
                    if (confirm('Are you sure you want to delete this?')) {
                        await ItemAPI.remove(this.data!.item_id);
                        Popups.add(`Removed ${this.data!.title}`)
                        router.navigate('/')
                    }
                }
            )
        ]))

        this.setup_title_view();
        this.setup_types_view();
        this.setup_attributes_view();
        this.setup_content_view();
        await this.render_comments();
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
        // let delete_button = this.querySelector('[name="delete_button"]')!;
        // delete_button.addEventListener('click', async () => {
        //     if (confirm("Are you sure you want to delete this?")) {
        //         await ItemAPI.remove(this.data!.item_id)
        //         Popups.add(`Removed ${this.data!.title}`);
        //         router.navigate('/')
        //     }
        // })
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
                    Popups.add(`Assigned type ${item} to ${this.data!.title}`)
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
                    Popups.add(`Unassigned type ${item} to ${this.data!.title}`)
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
        view.style.display = (content_visible) ? 'block' : 'none';
    }    

    async render_children() {
        let item = this.data!;
        let children_view = this.querySelector('item-list-view')! as ItemListView;

        let children = await API.item.related(item.item_id);

        children_view
            .enable_add_item(
                {Parent: [this.data!.title]},
                async () => {
                    children_view.only_render_items = true;
                    children_view.data = await API.item.children(item.item_id);
                }
            )
            .init(children);
        this.data!.children = children;
        children_view.style.display = (related_visible) ? 'block' : 'none';
    }

    async render_comments() {
        const list = this.querySelector('[name="item_comments_list"]') as HTMLDivElement;
        const form = this.querySelector('[name="item_comments_form"]') as HTMLFormElement;
        const timeInput = this.querySelector('[name="item_comments_time"]') as HTMLInputElement;
        const editorHost = this.querySelector('[name="item_comments_editor"]') as HTMLDivElement;
        const toggle = this.querySelector('[name="item_comments_toggle"]') as HTMLButtonElement;

        if (!list || !form || !timeInput || !editorHost || !toggle) {
            return;
        }

        if (toggle.dataset.bound !== "1") {
            toggle.dataset.bound = "1";
            toggle.addEventListener("click", () => {
                const isHidden = form.style.display === "none";
                form.style.display = isHidden ? "grid" : "none";
                toggle.innerText = isHidden ? "Close" : "Add";
            });
        }

        if (!timeInput.value) {
            const now = DateTime.now();
            timeInput.value = now.toFormat("HH:mm");
        }

        const loadEntries = async () => {
            list.innerHTML = "";
            let entries = [];
            try {
                entries = await API.comments.get_for_item(this.data!.item_id);
            } catch (e) {
                console.error(e);
                list.innerText = "Error loading comments.";
                return;
            }

            if (!entries || entries.length == 0) {
                list.innerText = "No comments.";
                return;
            }

            entries.forEach((entry) => {
                const row = document.createElement("div");
                row.classList.add("comment-entry");

                const time = DateTime.fromISO(entry.timestamp);
                const timeInput = document.createElement("input");
                timeInput.type = "datetime-local";
                timeInput.step = "60";
                timeInput.value = time.toFormat("yyyy-LL-dd'T'HH:mm");
                timeInput.classList.add("comment-time");
                timeInput.addEventListener("change", async () => {
                    const nextTime = DateTime.fromISO(timeInput.value);
                    if (!nextTime.isValid) {
                        return;
                    }
                    try {
                        await API.comments.update_entry(entry.comment_id, entry.content || "", nextTime);
                    } catch (e) {
                        console.error(e);
                    }
                });

                const metaRow = document.createElement("div");
                metaRow.classList.add("comment-meta");

                const actions = document.createElement("div");
                actions.classList.add("comment-actions");

                const comment = document.createElement("div");
                comment.classList.add("comment-editor");
                createZealotEditorView(comment, {
                    content: entry.content || "",
                    debounceMs: 500,
                    onUpdate: async (nextContent) => {
                        try {
                            entry.content = nextContent;
                            await API.comments.update_entry(entry.comment_id, nextContent);
                        } catch (e) {
                            console.error(e);
                        }
                    }
                });

                const del = document.createElement("button");
                del.innerText = "Delete";
                del.addEventListener("click", async () => {
                    try {
                        await API.comments.delete_entry(entry.comment_id);
                        await loadEntries();
                    } catch (e) {
                        console.error(e);
                    }
                });

                actions.appendChild(del);
                metaRow.appendChild(timeInput);
                metaRow.appendChild(actions);

                row.appendChild(metaRow);
                row.appendChild(comment);
                list.appendChild(row);
            });
        };

        if (form.dataset.bound !== "1") {
            form.dataset.bound = "1";
            let newContent = "";
            let editorView = createZealotEditorView(editorHost, {
                content: "",
                debounceMs: 200,
                onUpdate: (nextContent) => {
                    newContent = nextContent;
                }
            });
            form.addEventListener("submit", async (e) => {
                e.preventDefault();

                const [hour, minute] = (timeInput.value || "00:00").split(":").map(Number);
                const timestamp = DateTime.now().set({
                    hour: Number.isFinite(hour) ? hour : 0,
                    minute: Number.isFinite(minute) ? minute : 0,
                    second: 0,
                    millisecond: 0
                });

                const content = serializeZealotScript(editorView.state.doc);

                try {
                    await API.comments.add_entry(this.data!.item_id, timestamp, content);
                    newContent = "";
                    editorView.updateState(createZealotEditorState(""));
                    await loadEntries();
                } catch (e) {
                    console.error(e);
                }
            });
        }

        await loadEntries();
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
            setTimeout(() => {
                runner.run('Focus Edit')
            }, 200);
        })
        this.querySelector('button')?.focus();
    }
    
    public async LoadItem(title: string) {
        this.last_loaded_title = title;
        try {
            this.data = await ItemAPI.get_by_title(title);
        } catch (e) {
            // @ts-ignore
            this.data = null;
            // this.innerHTML = "<div class='error'>Error loading item</div>";
        }
    }

    public async LoadItemByID(item_id: number) {
        this.last_loaded_title = null;
        try {
            this.data = await ItemAPI.get(item_id);
        } catch (e) {
            this.innerHTML = "<div class='error'>Error loading item</div>"
        }
    }
}

customElements.define('item-screen', ItemScreen)

export default ItemScreen;
