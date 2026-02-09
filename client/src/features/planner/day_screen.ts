import { DateTime } from "luxon";
import API from "../../api/api";
import { router } from "../router/router";

import { HomeIcon, BackIcon, ForwardIcon, WeekIcon, MoonIcon, SunIcon, DocIcon } from "../../assets/asset_map";

import BaseElement from "../../shared/base_element";
import ButtonGroup, {ButtonDef} from "../../shared/button_group";
import type ItemListView from "../item/item_list_view";
import DragUtil from "../item/drag_helper";
import { CopyIcon } from "../../assets/asset_map";
import ItemSearchInline from "../item/item_search_inline";
import createZealotEditorView, {createZealotEditorState} from "../zealotscript/zealotscript_editor";
import { serializeZealotScript } from "../zealotscript/serializer";

class DailyPlannerScreen extends BaseElement<DateTime> {
    private current: any = {}

    async render() {
        let date = this.data!;
        this.current.date = this.data!;
        this.classList.add('center')
        this.innerHTML = `
        <h1>${date.toFormat(`EEEE - d MMMM yyyy`)}</h1>
        <item-list-view></item-list-view>
        <div class="repeat-planner">
            <h2>Repeats</h2>
            <div name="repeat_list" style="display: grid; gap: 8px;"></div>
        </div>
        <div class="comments-planner comments-section">
            <div style="display: grid; grid-template-columns: 1fr auto">
                <h2>Comments</h2>
                <button type="button" name="comments_toggle">Add</button>
            </div>
            <form name="comments_form" class="comments-form" style="display: none;">
                <div class="comments-form-row">
                    <item-search-inline name="comments_item_search"></item-search-inline>
                    <input type="time" name="comments_time">
                    <button type="submit">Log</button>
                </div>
                <div class="comments-form-editor" name="comments_editor"></div>
            </form>
            <div name="comments_list" style="display: grid; gap: 8px;"></div>
        </div>
        <div name="items" style="display: grid; grid-gap: 10px"></div>`

        this.prepend(new ButtonGroup().init([
            new ButtonDef(HomeIcon, "Today", () => {
                let today = DateTime.now()
                router.navigate(`/planner/daily/${today.toISODate()}`);
            }),
            new ButtonDef(BackIcon, "Previous Day", () => {
                let previous = date.minus({days: 1})
                router.navigate(`/planner/daily/${previous.toISODate()}`)
            }),
            new ButtonDef(ForwardIcon, "Next Day", () => {
                let next = date.plus({days: 1});
                router.navigate(`/planner/daily/${next.toISODate()}`)
            }),
            new ButtonDef(WeekIcon, `Week ${date.weekNumber} - ${date.year}`, () => {
                let week = date.toISOWeekDate()?.substring(0, 8);
                router.navigate(`/planner/weekly/${week}`)
            }),
            new ButtonDef(MoonIcon, `${date.monthLong} ${date.year}`, () => {
                let month_str = date.toFormat(`yyyy-MM`)
                router.navigate(`/planner/monthly/${month_str}`);
            }),
            new ButtonDef(SunIcon, `${date.year}`, () => {
                router.navigate(`/planner/annual/${date.year}`)
            }),
            new ButtonDef(DocIcon, 'Create Note', () => {
                router.navigate(`/item/${date!.toISODate()}`)
            }),
            new ButtonDef(CopyIcon, 'Copy Data as JSON', () => {
                navigator.clipboard.writeText(JSON.stringify(this.current));
            })
        ]))

        let items_view = this.querySelector('item-list-view')! as ItemListView;
        try {
            let items = await API.planner.get_items_on_day(date!)
            items_view
                .enable_add_item(
                    {
                        Date: date.toISODate(),
                        Status: "To Do",
                        Priority: 3,
                        Icon: ''
                    },
                    async () => {
                        items_view.only_render_items = true;
                        items_view.data = await API.planner.get_items_on_day(date!)
                    }
                )
                .init(items);
            this.current.assigned = items;
        } catch (e) {
            console.error(e)
        }

        DragUtil.setup_drop(this.querySelector('[title="Previous Day"]')!, 
            {"Date": this.data!.minus({days: 1}).toISODate()});
        DragUtil.setup_drop(this.querySelector('[title="Next Day"]')!, 
            {"Date": this.data!.plus({days: 1}).toISODate()});

        await this.render_repeats(date);
        await this.render_comments(date);
    }

    private async render_repeats(date: DateTime) {
        const container = this.querySelector('[name="repeat_list"]') as HTMLDivElement;
        if (!container) {
            return;
        }
        container.innerHTML = "";

        let repeats = [];
        try {
            repeats = await API.repeat.get_for_day(date);
        } catch (e) {
            console.error(e);
            container.innerText = "Error loading repeats.";
            return;
        }

        if (!repeats || repeats.length == 0) {
            container.innerText = "No repeats scheduled.";
            return;
        }

        const order = ["Morning", "Afternoon", "Evening", "Anytime"];
        const buckets: Record<string, typeof repeats> = {
            Morning: [],
            Afternoon: [],
            Evening: [],
            Anytime: []
        };

        repeats.forEach((entry) => {
            const raw = entry.item.attributes?.["Time of Day"];
            const label = typeof raw === "string" ? raw : "Anytime";
            if (label in buckets) {
                buckets[label].push(entry);
            } else {
                buckets.Anytime.push(entry);
            }
        });

        order.forEach((label) => {
            const group = buckets[label];
            if (!group || group.length == 0) {
                return;
            }

            const header = document.createElement("div");
            header.classList.add("repeat-group");
            header.innerText = label;
            container.appendChild(header);

            group.forEach((entry) => {
                const row = document.createElement("div");
                row.classList.add("repeat-row");
                row.style.display = "grid";
                row.style.gridTemplateColumns = "1.5em 1fr 2fr 8em";
                row.style.alignItems = "center";
                row.style.gap = "8px";

                const checkbox = document.createElement("input");
                checkbox.type = "checkbox";
                checkbox.checked = entry.status !== "Not Completed";

                const title = document.createElement("div");
                title.classList.add("repeat-title");
                title.innerText = (entry.item.attributes!['Icon'] || "") + " " + entry.item.title;
                title.style.cursor = "pointer";
                title.addEventListener("click", () => {
                    router.navigate(`/item_id/${entry.item.item_id}`);
                });

                const comment = document.createElement("input");
                comment.type = "text";
                comment.placeholder = "Comment";
                comment.value = entry.comment || "";

                const statusLabel = document.createElement("div");
                statusLabel.classList.add("repeat-status");
                statusLabel.innerText = entry.status;

                const save = async () => {
                    const status = checkbox.checked ? "Complete" : "Not Completed";
                    const commentText = comment.value || "";
                    statusLabel.innerText = status;
                    try {
                        await API.repeat.set_status(entry.item.item_id, date, status, commentText);
                    } catch (e) {
                        console.error(e);
                    }
                };

                checkbox.addEventListener("change", save);
                comment.addEventListener("blur", save);

                row.appendChild(checkbox);
                row.appendChild(title);
                row.appendChild(comment);
                row.appendChild(statusLabel);
                container.appendChild(row);
            });
        });
        this.current.repeats = repeats;
    }

    private async render_comments(date: DateTime) {
        const list = this.querySelector('[name="comments_list"]') as HTMLDivElement;
        const form = this.querySelector('[name="comments_form"]') as HTMLFormElement;
        const itemSearch = this.querySelector('[name="comments_item_search"]') as ItemSearchInline;
        const timeInput = this.querySelector('[name="comments_time"]') as HTMLInputElement;
        const editorHost = this.querySelector('[name="comments_editor"]') as HTMLDivElement;
        const toggle = this.querySelector('[name="comments_toggle"]') as HTMLButtonElement;

        if (!list || !form || !itemSearch || !timeInput || !editorHost || !toggle) {
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

        list.innerHTML = "";

        if (itemSearch.dataset.bound !== "1") {
            itemSearch.dataset.bound = "1";
            itemSearch.init({
                on_search: async (term: string) => {
                    return await API.item.search(term);
                },
                on_match_text: (item) => item.title,
                on_select: (item) => {
                    itemSearch.setSelected(item);
                },
                on_make_view: (item) => {
                    const div = document.createElement("div");
                    div.innerText = `${item.attributes?.["Icon"] || ""} ${item.title}`.trim();
                    return div;
                }
            });
        }

        const searchInput = itemSearch.querySelector('[name="search"]') as HTMLInputElement | null;
        if (searchInput && !searchInput.placeholder) {
            searchInput.placeholder = "Search items...";
        }

        if (!timeInput.value) {
            const now = DateTime.now();
            timeInput.value = now.toFormat("HH:mm");
        }

        const loadEntries = async () => {
            list.innerHTML = "";
            let entries = [];
            try {
                entries = await API.comment.get_for_day(date);
            } catch (e) {
                console.error(e);
                list.innerText = "Error loading comments.";
                return;
            }

            if (!entries || entries.length == 0) {
                list.innerText = "No comments.";
                return;
            }
            entries = entries.sort((a, b) => {
                const aTime = DateTime.fromISO(a.timestamp).toMillis();
                const bTime = DateTime.fromISO(b.timestamp).toMillis();
                return bTime - aTime;
            });
            this.current.comments = entries;

            entries.forEach((entry) => {
                const row = document.createElement("div");
                row.classList.add("comment-entry");

                const time = DateTime.fromISO(entry.timestamp);
                const timeInput = document.createElement("input");
                timeInput.type = "time";
                timeInput.value = time.toFormat("HH:mm");
                timeInput.classList.add("comment-time");
                timeInput.addEventListener("change", async () => {
                    const [hour, minute] = (timeInput.value || "00:00").split(":").map(Number);
                    const nextTimestamp = time.set({
                        hour: Number.isFinite(hour) ? hour : 0,
                        minute: Number.isFinite(minute) ? minute : 0,
                        second: 0,
                        millisecond: 0
                    });
                    try {
                        await API.comment.update_entry(entry.comment_id, entry.content || "", nextTimestamp);
                    } catch (e) {
                        console.error(e);
                    }
                });

                const title = document.createElement("div");
                title.innerText = entry.item.title;
                title.style.cursor = "pointer";
                title.classList.add("comment-item");
                title.addEventListener("click", () => {
                    router.navigate(`/item_id/${entry.item.item_id}`);
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
                    handleTab: true,
                    onUpdate: async (nextContent) => {
                        try {
                            entry.content = nextContent;
                            await API.comment.update_entry(entry.comment_id, nextContent);
                        } catch (e) {
                            console.error(e);
                        }
                    }
                });

                const del = document.createElement("button");
                del.innerText = "Delete";
                del.addEventListener("click", async () => {
                    try {
                        await API.comment.delete_entry(entry.comment_id);
                        await loadEntries();
                    } catch (e) {
                        console.error(e);
                    }
                });

                actions.appendChild(del);
                metaRow.appendChild(timeInput);
                metaRow.appendChild(title);
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
                handleTab: true,
                onUpdate: (nextContent) => {
                    newContent = nextContent;
                }
            });
            form.addEventListener("submit", async (e) => {
                e.preventDefault();
                const selectedItem = itemSearch.value;
                if (!selectedItem) {
                    return;
                }

                const [hour, minute] = (timeInput.value || "00:00").split(":").map(Number);
                const timestamp = date.set({
                    hour: Number.isFinite(hour) ? hour : 0,
                    minute: Number.isFinite(minute) ? minute : 0,
                    second: 0,
                    millisecond: 0
                });

                const content = serializeZealotScript(editorView.state.doc);

                try {
                    await API.comment.add_entry(selectedItem.item_id, timestamp, content);
                    newContent = "";
                    editorView.updateState(createZealotEditorState(""));
                    itemSearch.clear();
                    await loadEntries();
                } catch (e) {
                    console.error(e);
                }
            });
        }

        await loadEntries();
    }
}

customElements.define('daily-planner-screen', DailyPlannerScreen)

export default DailyPlannerScreen;
