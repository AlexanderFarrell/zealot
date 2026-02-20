import { DateTime } from "luxon";
import API from "../../api/api";
import type { CommentEntry } from "../../api/comment";
import { router } from "../router/router";

import BaseElement from "../../shared/base_element";
import ItemSearchInline from "../item/item_search_inline";
import { createZealotEditorState, createZealotEditorView } from "../zealotscript/zealotscript_editor";
import { serializeZealotScript } from "../zealotscript/serializer";
import {
	createCommentEditor,
	createDeleteButton,
	emitCommentsLoaded,
	sortComments
} from "./comment_shared";

export interface CommentsViewScope {
	date?: DateTime;
	itemID?: number;
}

class CommentsView extends BaseElement<CommentsViewScope> {
	async render() {
		const scope = this.data || {};
		const scopedDate = scope.date ?? null;
		const scopedItemID = scope.itemID ?? null;
		const canSelectItem = scopedItemID == null;
		const useTimeOnly = scopedDate != null;
		const rowClass = useTimeOnly ? "comments-form-row" : "comments-form-row comments-form-row-datetime";

		this.classList.add("comments-section");
		if (scopedItemID != null) {
			this.classList.add("item-comments");
		} else {
			this.classList.remove("item-comments");
		}

		this.innerHTML = `
			<div style="display: grid; grid-template-columns: 1fr auto">
				<h2>Comments</h2>
				<button type="button" name="comments_toggle">Add</button>
			</div>
			<form name="comments_form" class="comments-form" style="display: none;">
				<div class="${rowClass}">
					${canSelectItem ? '<item-search-inline name="comments_item_search"></item-search-inline>' : ""}
					<input ${useTimeOnly ? 'type="time"' : 'type="datetime-local" step="60"'} name="comments_time">
					<button type="submit">Log</button>
				</div>
				<div class="comments-form-editor" name="comments_editor"></div>
			</form>
			<div name="comments_list" style="display: grid; gap: 8px;"></div>
		`;

		await this.renderComments(scope);
	}

	private async renderComments(scope: CommentsViewScope) {
		const scopedDate = scope.date ?? null;
		const scopedItemID = scope.itemID ?? null;
		const canSelectItem = scopedItemID == null;
		const useTimeOnly = scopedDate != null;
		const isUnscoped = scopedDate == null && scopedItemID == null;

		const list = this.querySelector('[name="comments_list"]') as HTMLDivElement;
		const form = this.querySelector('[name="comments_form"]') as HTMLFormElement;
		const itemSearch = this.querySelector('[name="comments_item_search"]') as ItemSearchInline | null;
		const timeInput = this.querySelector('[name="comments_time"]') as HTMLInputElement;
		const editorHost = this.querySelector('[name="comments_editor"]') as HTMLDivElement;
		const toggle = this.querySelector('[name="comments_toggle"]') as HTMLButtonElement;

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

		let activeItemID: number | null = scopedItemID;

		if (canSelectItem && itemSearch) {
			if (itemSearch.dataset.bound !== "1") {
				itemSearch.dataset.bound = "1";
				itemSearch.init({
					on_search: async (term: string) => {
						return await API.item.search(term);
					},
					on_match_text: (item) => item.title,
					on_select: (item) => {
						itemSearch.setSelected(item);
						if (isUnscoped) {
							activeItemID = item.item_id;
							void loadEntries();
						}
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
		}

		if (!timeInput.value) {
			const now = DateTime.now();
			timeInput.value = useTimeOnly
				? now.toFormat("HH:mm")
				: now.toFormat("yyyy-LL-dd'T'HH:mm");
		}

		const loadEntries = async () => {
			list.innerHTML = "";
			let entries: CommentEntry[] = [];
			try {
				if (scopedDate != null) {
					entries = await API.comment.get_for_day(scopedDate);
					if (activeItemID != null) {
						entries = entries.filter((entry) => entry.item.item_id === activeItemID);
					}
				} else if (activeItemID != null) {
					entries = await API.comment.get_for_item(activeItemID);
				} else {
					emitCommentsLoaded(this, []);
					list.innerText = "Select an item to load comments.";
					return;
				}
			} catch (e) {
				console.error(e);
				list.innerText = "Error loading comments.";
				return;
			}

			if (!entries || entries.length == 0) {
				emitCommentsLoaded(this, []);
				list.innerText = "No comments.";
				return;
			}

			entries = sortComments(entries);
			emitCommentsLoaded(this, entries);

			const showItemTitle = scopedItemID == null;
			entries.forEach((entry) => {
				const row = document.createElement("div");
				row.classList.add("comment-entry");

				const entryTimestamp = DateTime.fromISO(entry.timestamp);
				const entryTimeInput = document.createElement("input");
				if (useTimeOnly) {
					entryTimeInput.type = "time";
					entryTimeInput.value = entryTimestamp.toFormat("HH:mm");
				} else {
					entryTimeInput.type = "datetime-local";
					entryTimeInput.step = "60";
					entryTimeInput.value = entryTimestamp.toFormat("yyyy-LL-dd'T'HH:mm");
				}
				entryTimeInput.classList.add("comment-time");
				entryTimeInput.addEventListener("change", async () => {
					let nextTime: DateTime;
					if (useTimeOnly) {
						const [hour, minute] = (entryTimeInput.value || "00:00").split(":").map(Number);
						nextTime = entryTimestamp.set({
							hour: Number.isFinite(hour) ? hour : 0,
							minute: Number.isFinite(minute) ? minute : 0,
							second: 0,
							millisecond: 0
						});
					} else {
						nextTime = DateTime.fromISO(entryTimeInput.value);
						if (!nextTime.isValid) {
							return;
						}
					}
					try {
						await API.comment.update_entry(entry.comment_id, entry.content || "", nextTime);
					} catch (e) {
						console.error(e);
					}
				});

				const metaRow = document.createElement("div");
				metaRow.classList.add("comment-meta");
				if (!showItemTitle) {
					metaRow.classList.add("comment-meta-no-title");
				}

				const actions = document.createElement("div");
				actions.classList.add("comment-actions");
				actions.appendChild(createDeleteButton(entry.comment_id, loadEntries));

				metaRow.appendChild(entryTimeInput);
				if (showItemTitle) {
					const title = document.createElement("div");
					title.innerText = entry.item.title;
					title.style.cursor = "pointer";
					title.classList.add("comment-item");
					title.addEventListener("click", () => {
						router.navigate(`/item_id/${entry.item.item_id}`);
					});
					metaRow.appendChild(title);
				}
				metaRow.appendChild(actions);

				row.appendChild(metaRow);
				row.appendChild(createCommentEditor(entry));
				list.appendChild(row);
			});
		};

		if (form.dataset.bound !== "1") {
			form.dataset.bound = "1";
			const editorView = createZealotEditorView(editorHost, {
				content: "",
				debounceMs: 200,
				handleTab: true,
				onUpdate: () => {}
			});
			form.addEventListener("submit", async (e) => {
				e.preventDefault();

				let selectedItemID = scopedItemID;
				if (selectedItemID == null) {
					const selectedItem = itemSearch?.value;
					if (!selectedItem) {
						return;
					}
					selectedItemID = selectedItem.item_id;
					if (isUnscoped) {
						activeItemID = selectedItemID;
					}
				}

				let timestamp: DateTime;
				if (useTimeOnly) {
					const [hour, minute] = (timeInput.value || "00:00").split(":").map(Number);
					const baseDate = scopedDate || DateTime.now();
					timestamp = baseDate.set({
						hour: Number.isFinite(hour) ? hour : 0,
						minute: Number.isFinite(minute) ? minute : 0,
						second: 0,
						millisecond: 0
					});
				} else {
					const parsed = DateTime.fromISO(timeInput.value);
					if (!parsed.isValid) {
						return;
					}
					timestamp = parsed;
				}

				const content = serializeZealotScript(editorView.state.doc);
				try {
					await API.comment.add_entry(selectedItemID, timestamp, content);
					editorView.updateState(createZealotEditorState(""));
					if (canSelectItem && itemSearch && !isUnscoped) {
						itemSearch.clear();
					}
					await loadEntries();
				} catch (e) {
					console.error(e);
				}
			});
		}

		await loadEntries();
	}
}

customElements.define("comments-view", CommentsView);

export default CommentsView;
