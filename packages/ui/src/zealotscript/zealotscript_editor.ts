import { EditorState, type Transaction } from "prosemirror-state";
import { EditorView } from "prosemirror-view";
import { buildKeymap } from "prosemirror-example-setup";
import { keymap } from "prosemirror-keymap";
import { baseKeymap } from "prosemirror-commands";
import { dropCursor } from "prosemirror-dropcursor";
import { gapCursor } from "prosemirror-gapcursor";
import { history } from "prosemirror-history";
import { InputRule, inputRules, textblockTypeInputRule, wrappingInputRule } from "prosemirror-inputrules";
import { liftListItem, sinkListItem } from "prosemirror-schema-list";
import type { MarkType, Schema } from "prosemirror-model";
import ZealotSchema from "./schema";
import { parseZealotScript } from "./parser";
import { serializeZealotScript } from "./serializer";
import { insertTable, insertAdmonition, insertYoutubeEmbed, insertMermaidBlock, extractYouTubeVideoId, insertDetails, insertSpoiler, insertColumns, insertTabs } from "./commands";
import { TabsView } from "./zealotscript_view";
import { EMOJI_MAP, lookupEmoji } from "./emoji_map";
import { getNavigator } from "@websoil/engine";
import { ItemSearchInline } from "../views/item_search_inline";
import type { Item } from "@zealot/domain/src/item";

type PMCommand = (state: EditorState, dispatch?: (tr: Transaction) => void) => boolean;

export class TaskListItemView {
	dom: HTMLElement;
	contentDOM: HTMLElement;

	constructor(
		private node: import("prosemirror-model").Node,
		private view: EditorView,
		private getPos: () => number | undefined,
	) {
		const li = document.createElement("li");
		const checked: boolean | null = node.attrs.checked ?? null;
		if (checked !== null) {
			li.className = "zealot-task-item";
			li.setAttribute("data-checked", String(checked));
		}

		if (checked !== null) {
			const input = document.createElement("input");
			input.type = "checkbox";
			input.className = "zealot-task-checkbox";
			input.checked = checked;
			input.addEventListener("mousedown", (e) => e.preventDefault());
			input.addEventListener("change", () => {
				const pos = this.getPos();
				if (pos === undefined) return;
				const tr = this.view.state.tr.setNodeMarkup(pos, undefined, {
					...this.node.attrs,
					checked: input.checked,
				});
				this.view.dispatch(tr);
			});
			li.appendChild(input);
		}

		const content = document.createElement("span");
		content.className = "zealot-task-content";
		li.appendChild(content);

		this.dom = li;
		this.contentDOM = content;
	}

	update(node: import("prosemirror-model").Node): boolean {
		if (node.type !== this.node.type) return false;
		this.node = node;
		const checked: boolean | null = node.attrs.checked ?? null;
		if (checked !== null) {
			this.dom.setAttribute("data-checked", String(checked));
			const input = this.dom.querySelector<HTMLInputElement>("input[type=checkbox]");
			if (input) input.checked = checked;
		}
		return true;
	}
}

export class YoutubeEmbedView {
	dom: HTMLElement;

	constructor(node: import("prosemirror-model").Node) {
		const videoId = (node.attrs.videoId as string) || "";
		const wrapper = document.createElement("div");
		wrapper.className = "zealot-youtube-embed";
		wrapper.contentEditable = "false";
		if (videoId) {
			const iframe = document.createElement("iframe");
			iframe.src = `https://www.youtube.com/embed/${videoId}`;
			iframe.allow = "accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share";
			iframe.allowFullscreen = true;
			iframe.setAttribute("frameborder", "0");
			wrapper.appendChild(iframe);
		} else {
			wrapper.textContent = "YouTube embed (no video ID)";
		}
		this.dom = wrapper;
	}
}

const buildInputRules = (schema: Schema) => {
	const rules: InputRule[] = [];

	const headingType = schema.nodes["heading"];
	if (headingType) {
		rules.push(textblockTypeInputRule(/^(#{1,6})\s$/, headingType, (match) => ({
			level: (match[1] ?? "#").length,
		})));
	}

	const codeBlockType = schema.nodes["code_block"];
	if (codeBlockType) {
		rules.push(textblockTypeInputRule(/^```([A-Za-z0-9_+-]*)\s$/, codeBlockType, (match) => ({
			language: (match[1] ?? "").trim(),
		})));
	}

	const blockquoteType = schema.nodes["blockquote"];
	if (blockquoteType) {
		rules.push(wrappingInputRule(/^\s*>\s$/, blockquoteType));
	}

	const orderedListType = schema.nodes["ordered_list"];
	if (orderedListType) {
		rules.push(wrappingInputRule(
			/^(\d+)\.\s$/,
			orderedListType,
			(match) => ({ order: +(match[1] ?? "1") }),
			(match, node) => node.childCount + node.attrs.order === +(match[1] ?? "1")
		));
	}

	const bulletListType = schema.nodes["bullet_list"];
	if (bulletListType) {
		rules.push(wrappingInputRule(/^\s*([-+*])\s$/, bulletListType));
	}

	const listItemType = schema.nodes["list_item"];
	if (listItemType) {
		rules.push(new InputRule(/^- \[([xX ])\]\s$/, (state, match, start, end) => {
			const checked = (match[1] ?? " ") !== " ";
			const { $from } = state.selection;
			const listItem = state.schema.nodes["list_item"];
			const bulletList = state.schema.nodes["bullet_list"];
			const paragraph = state.schema.nodes["paragraph"];
			if (!listItem || !bulletList || !paragraph) return null;
			const item = listItem.create({ checked }, paragraph.create());
			const list = bulletList.create(null, item);
			const from = $from.before($from.depth === 0 ? 1 : $from.depth);
			return state.tr.replaceWith(from, end, list);
		}));
	}

	const addMarkRule = (markType: MarkType | undefined, regex: RegExp) => {
		if (!markType) return;
		rules.push(new InputRule(regex, (state, match, start, end) => {
			const inner = match[1];
			if (!inner || typeof inner !== "string") return null;
			const node = schema.text(inner, [markType.create()]);
			return state.tr.replaceWith(start, end, node);
		}));
	};

	addMarkRule(schema.marks["strong"], /\*\*([^*\n]+)\*\*$/);
	addMarkRule(schema.marks["em"], /(?<!\*)\*([^*\n]+)\*$/);
	addMarkRule(schema.marks["strike"], /~~([^~\n]+)~~$/);
	addMarkRule(schema.marks["code"], /`([^`\n]+)`$/);
	addMarkRule(schema.marks["underline"], /(?<![A-Za-z0-9_])_([^_\n]+)_$/);

	rules.push(new InputRule(/:([a-z0-9_+\-]+):$/, (state, match, start, end) => {
		const shortcode = match[1];
		if (!shortcode) return null;
		const emoji = lookupEmoji(shortcode);
		if (!emoji) return null;
		return state.tr.replaceWith(start, end, schema.text(emoji));
	}));

	const linkMark = schema.marks["link"];
	if (linkMark) {
		rules.push(new InputRule(/\[([^\]]+)\]\(([^)]+)\)\s$/, (state, match, start, end) => {
			const label = match[1];
			const href = match[2];
			if (!label || !href) return null;
			const mark = linkMark.create({ href });
			const text = schema.text(label, [mark]);
			const tr = state.tr.replaceWith(start, end, text);
			const insertPos = tr.mapping.map(start) + text.nodeSize;
			return tr.insertText(" ", insertPos);
		}));
	}

	return inputRules({ rules });
};

const buildPlugins = (schema: Schema) => {
	const linkMark = schema.marks["link"];

	const addLink: PMCommand = (state, dispatch) => {
		const { from, to } = state.selection;
		const href = window.prompt("Add Link URL");
		if (!href) return false;
		if (dispatch && linkMark) {
			dispatch(state.tr.addMark(from, to, linkMark.create({ href })));
		}
		return true;
	};

	const removeLink: PMCommand = (state, dispatch) => {
		const { from, to } = state.selection;
		if (dispatch && linkMark) {
			dispatch(state.tr.removeMark(from, to, linkMark));
		}
		return true;
	};

	return [
		keymap(buildKeymap(schema)),
		keymap(baseKeymap),
		dropCursor(),
		gapCursor(),
		history(),
		keymap({ "Mod-k": addLink, "Mod-Shift-k": removeLink }),
		buildInputRules(schema),
	];
};

const isInList = (state: EditorState): boolean => {
	const listItemType = state.schema.nodes["list_item"];
	if (!listItemType) return false;
	const { $from } = state.selection;
	for (let depth = $from.depth; depth > 0; depth--) {
		if ($from.node(depth).type === listItemType) return true;
	}
	return false;
};

function detectEmojiTrigger(state: EditorState): { from: number; query: string } | null {
	const { $from } = state.selection;
	if ($from.depth === 0) return null;
	const text = $from.parent.textBetween(0, $from.parentOffset, null, "\0");
	const lastColon = text.lastIndexOf(":");
	if (lastColon === -1) return null;
	const query = text.slice(lastColon + 1);
	if (query.includes(":") || query.includes(" ") || query.includes("\n")) return null;
	if (!/^[a-z0-9_+\-]*$/.test(query)) return null;
	if (query.length === 0) return null;
	return { from: $from.start() + lastColon, query };
}

function detectWikilinkTrigger(state: EditorState): { from: number; query: string } | null {
	const { $from } = state.selection;
	if ($from.depth === 0) return null;
	const text = $from.parent.textBetween(0, $from.parentOffset, null, "\0");
	const lastBrackets = text.lastIndexOf("[[");
	if (lastBrackets === -1) return null;
	const query = text.slice(lastBrackets + 2);
	if (query.includes("]]") || query.includes("\n")) return null;
	return { from: $from.start() + lastBrackets, query };
}

export class ZealotScriptEditor extends HTMLElement {
	private _view: EditorView | null = null;
	private _saveTimer: number | null = null;
	private _lastValue = "";
	private _wikilinkPickerEl: HTMLDivElement | null = null;
	private _wikilinkPickerSearch: ItemSearchInline | null = null;
	private _wikilinkPickerFrom: number | null = null;
	private _outsideClickListener: ((e: MouseEvent) => void) | null = null;
	private _emojiPickerEl: HTMLDivElement | null = null;
	private _emojiPickerFrom: number | null = null;
	private _emojiOutsideClickListener: ((e: MouseEvent) => void) | null = null;

	private _showWikilinkPicker(from: number, query: string): void {
		if (!this._view) return;
		this._wikilinkPickerFrom = from;

		if (!this._wikilinkPickerEl) {
			const wrapper = document.createElement("div");
			wrapper.className = "zealotscript-wikilink-picker";
			const search = new ItemSearchInline();
			search.placeholder = "Search items…";
			search.OnSelect = (item) => this._onWikilinkSelect(item);
			wrapper.appendChild(search);
			document.body.appendChild(wrapper);
			this._wikilinkPickerEl = wrapper;
			this._wikilinkPickerSearch = search;

			this._outsideClickListener = (e: MouseEvent) => {
				if (
					this._wikilinkPickerEl &&
					!this._wikilinkPickerEl.contains(e.target as Node) &&
					!this.contains(e.target as Node)
				) {
					this._hideWikilinkPicker();
				}
			};
			document.addEventListener("mousedown", this._outsideClickListener);
		}

		const coords = this._view.coordsAtPos(from);
		const el = this._wikilinkPickerEl;
		el.style.position = "fixed";
		el.style.top = `${coords.bottom + 4}px`;
		el.style.left = `${coords.left}px`;

		this._wikilinkPickerSearch?.setQuery(query);
	}

	private _updateWikilinkPicker(from: number, query: string): void {
		if (!this._wikilinkPickerEl) {
			this._showWikilinkPicker(from, query);
			return;
		}
		this._wikilinkPickerFrom = from;
		if (!this._view) return;
		const coords = this._view.coordsAtPos(from);
		this._wikilinkPickerEl.style.top = `${coords.bottom + 4}px`;
		this._wikilinkPickerEl.style.left = `${coords.left}px`;
		this._wikilinkPickerSearch?.setQuery(query);
	}

	private _hideWikilinkPicker(): void {
		if (this._outsideClickListener) {
			document.removeEventListener("mousedown", this._outsideClickListener);
			this._outsideClickListener = null;
		}
		this._wikilinkPickerEl?.remove();
		this._wikilinkPickerEl = null;
		this._wikilinkPickerSearch = null;
		this._wikilinkPickerFrom = null;
	}

	private _onWikilinkSelect(item: Item): void {
		if (!this._view || this._wikilinkPickerFrom === null) return;
		const { state } = this._view;
		const from = this._wikilinkPickerFrom;
		const to = state.selection.from;
		const linkMarkType = state.schema.marks["link"];
		if (!linkMarkType) return;
		const mark = linkMarkType.create({ href: `zealot://item/${item.Title}` });
		const linkText = state.schema.text(item.DisplayTitle, [mark]);
		const tr = state.tr
			.replaceWith(from, to, linkText)
			.insertText(" ", from + linkText.nodeSize);
		this._view.dispatch(tr);
		this._hideWikilinkPicker();
		this._view.focus();
	}

	private _showEmojiPicker(from: number, query: string): void {
		if (!this._view) return;
		this._emojiPickerFrom = from;

		if (!this._emojiPickerEl) {
			const wrapper = document.createElement("div");
			wrapper.className = "zealotscript-emoji-picker";
			document.body.appendChild(wrapper);
			this._emojiPickerEl = wrapper;

			this._emojiOutsideClickListener = (e: MouseEvent) => {
				if (
					this._emojiPickerEl &&
					!this._emojiPickerEl.contains(e.target as Node) &&
					!this.contains(e.target as Node)
				) {
					this._hideEmojiPicker();
				}
			};
			document.addEventListener("mousedown", this._emojiOutsideClickListener);
		}

		this._updateEmojiPickerContent(from, query);
	}

	private _updateEmojiPickerContent(from: number, query: string): void {
		const el = this._emojiPickerEl;
		if (!el || !this._view) return;

		const coords = this._view.coordsAtPos(from);
		el.style.position = "fixed";
		el.style.top = `${coords.bottom + 4}px`;
		el.style.left = `${coords.left}px`;

		const matches = Object.entries(EMOJI_MAP)
			.filter(([k]) => k.startsWith(query))
			.slice(0, 8);

		el.innerHTML = "";
		if (matches.length === 0) {
			this._hideEmojiPicker();
			return;
		}

		for (const [shortcode, emoji] of matches) {
			const btn = document.createElement("button");
			btn.type = "button";
			btn.className = "zealotscript-emoji-option";
			btn.textContent = `${emoji} ${shortcode}`;
			btn.addEventListener("mousedown", (e) => {
				e.preventDefault();
				this._insertEmoji(emoji);
			});
			el.appendChild(btn);
		}
	}

	private _insertEmoji(emoji: string): void {
		if (!this._view || this._emojiPickerFrom === null) return;
		const { state } = this._view;
		const from = this._emojiPickerFrom;
		const to = state.selection.from;
		const tr = state.tr.replaceWith(from, to, state.schema.text(emoji));
		this._view.dispatch(tr);
		this._hideEmojiPicker();
		this._view.focus();
	}

	private _hideEmojiPicker(): void {
		if (this._emojiOutsideClickListener) {
			document.removeEventListener("mousedown", this._emojiOutsideClickListener);
			this._emojiOutsideClickListener = null;
		}
		this._emojiPickerEl?.remove();
		this._emojiPickerEl = null;
		this._emojiPickerFrom = null;
	}

	private _buildToolbar(): HTMLElement {
		const bar = document.createElement("div");
		bar.className = "zealotscript-toolbar";

		const btn = (label: string, title: string, onClick: () => void): HTMLButtonElement => {
			const b = document.createElement("button");
			b.type = "button";
			b.textContent = label;
			b.title = title;
			b.addEventListener("mousedown", (e) => {
				e.preventDefault();
				onClick();
			});
			return b;
		};

		const runCommand = (cmd: (state: import("prosemirror-state").EditorState, dispatch?: (tr: import("prosemirror-state").Transaction) => void) => boolean) => {
			if (!this._view) return;
			cmd(this._view.state, this._view.dispatch.bind(this._view));
			this._view.focus();
		};

		const setHeading = (level: number) => {
			if (!this._view) return;
			const { state, dispatch } = this._view;
			const headingType = state.schema.nodes["heading"];
			if (!headingType) return;
			dispatch(state.tr.setBlockType(state.selection.from, state.selection.to, headingType, { level }));
			this._view.focus();
		};

		const toggleMark = (markName: string) => {
			if (!this._view) return;
			const { state } = this._view;
			const markType = state.schema.marks[markName];
			if (!markType) return;
			const { from, to } = state.selection;
			const hasMark = state.doc.rangeHasMark(from, to, markType);
			const tr = hasMark
				? state.tr.removeMark(from, to, markType)
				: state.tr.addMark(from, to, markType.create());
			this._view.dispatch(tr);
			this._view.focus();
		};

		bar.appendChild(btn("H1", "Heading 1", () => setHeading(1)));
		bar.appendChild(btn("H2", "Heading 2", () => setHeading(2)));
		bar.appendChild(btn("H3", "Heading 3", () => setHeading(3)));

		const sep1 = document.createElement("span");
		sep1.className = "zealotscript-toolbar-sep";
		bar.appendChild(sep1);

		bar.appendChild(btn("B", "Bold (Ctrl+B)", () => toggleMark("strong")));
		bar.appendChild(btn("I", "Italic (Ctrl+I)", () => toggleMark("em")));
		bar.appendChild(btn("S", "Strikethrough", () => toggleMark("strike")));
		bar.appendChild(btn("U", "Underline", () => toggleMark("underline")));
		bar.appendChild(btn("</>", "Inline Code", () => toggleMark("code")));

		const sep2 = document.createElement("span");
		sep2.className = "zealotscript-toolbar-sep";
		bar.appendChild(sep2);

		bar.appendChild(btn("☑ Task", "Insert Task List", () => {
			if (!this._view) return;
			const { state, dispatch } = this._view;
			const listItem = state.schema.nodes["list_item"];
			const bulletList = state.schema.nodes["bullet_list"];
			const paragraph = state.schema.nodes["paragraph"];
			if (!listItem || !bulletList || !paragraph) return;
			const item = listItem.create({ checked: false }, paragraph.create());
			const list = bulletList.create(null, item);
			const { $from } = state.selection;
			const insertPos = $from.before($from.depth === 0 ? 1 : $from.depth);
			dispatch(state.tr.replaceWith(insertPos, $from.after($from.depth === 0 ? 1 : $from.depth), list));
			this._view.focus();
		}));
		bar.appendChild(btn("Table", "Insert Table", () => runCommand(insertTable)));
		bar.appendChild(btn("Note", "Insert Note", () => runCommand(insertAdmonition("note"))));
		bar.appendChild(btn("Warning", "Insert Warning", () => runCommand(insertAdmonition("warning"))));
		bar.appendChild(btn("Tip", "Insert Tip", () => runCommand(insertAdmonition("tip"))));
		bar.appendChild(btn("▶ YouTube", "Insert YouTube Video", () => {
			const input = window.prompt("YouTube URL or video ID:");
			if (!input) return;
			const videoId = extractYouTubeVideoId(input.trim());
			if (videoId) runCommand(insertYoutubeEmbed(videoId));
		}));
		bar.appendChild(btn("◇ Mermaid", "Insert Mermaid Diagram", () => runCommand(insertMermaidBlock)));
		bar.appendChild(btn("◇ Details", "Insert Details block", () => {
			const summary = window.prompt("Summary text:", "Details") || "Details";
			runCommand(insertDetails(summary));
		}));
		bar.appendChild(btn("◇ Spoiler", "Insert Spoiler block", () => runCommand(insertSpoiler)));
		bar.appendChild(btn("⊟ Columns", "Insert 2-column layout", () => runCommand(insertColumns(2))));
		bar.appendChild(btn("⊞ Tabs", "Insert Tabs", () => runCommand(insertTabs(["Tab 1", "Tab 2"]))));

		return bar;
	}

	connectedCallback() {
		if (this._view) return;

		this.appendChild(this._buildToolbar());

		const editorContainer = document.createElement("div");
		editorContainer.className = "zealotscript-editor-container";
		this.appendChild(editorContainer);

		const state = EditorState.create({
			schema: ZealotSchema,
			doc: parseZealotScript(ZealotSchema, this._lastValue),
			plugins: buildPlugins(ZealotSchema),
		});

		this._view = new EditorView(editorContainer, {
			state,
			nodeViews: {
				youtube_embed: (node) => new YoutubeEmbedView(node),
				list_item: (node, view, getPos) => new TaskListItemView(node, view, getPos),
				tabs: (node) => new TabsView(node),
			},
			handleKeyDown: (_view, event) => {
				if (event.key === "Escape" && (this._wikilinkPickerEl || this._emojiPickerEl)) {
					event.preventDefault();
					this._hideWikilinkPicker();
					this._hideEmojiPicker();
					return true;
				}
				if (event.key !== "Tab") return false;
				event.preventDefault();
				const listItemType = _view.state.schema.nodes["list_item"];
				if (listItemType && isInList(_view.state)) {
					const command = event.shiftKey
						? liftListItem(listItemType)
						: sinkListItem(listItemType);
					if (command(_view.state, _view.dispatch)) return true;
				}
				_view.dispatch(_view.state.tr.insertText("\t"));
				return true;
			},
			handleClick: (_view, _pos, event) => {
				if (event.button !== 0) return false;
				const target = event.target as HTMLElement | null;

				const dateRef = target?.closest<HTMLElement>("span[data-date-ref]");
				if (dateRef) {
					const raw = dateRef.getAttribute("data-date-ref") ?? "";
					const kind = dateRef.getAttribute("data-date-kind") ?? "day";
					event.preventDefault();
					event.stopPropagation();
					if (kind === "day") getNavigator().openPlanner("daily", raw);
					else if (kind === "week") getNavigator().openPlanner("weekly", raw);
					else getNavigator().openPlanner("annual", raw);
					return true;
				}

				const anchor = target?.closest("a[href]") as HTMLAnchorElement | null;
				if (!anchor) return false;
				const href = anchor.getAttribute("href") || "";
				if (href.trim().length === 0) return false;
				event.preventDefault();
				event.stopPropagation();
				if (href.startsWith("zealot://item/")) {
					getNavigator().openItem(decodeURIComponent(href.slice("zealot://item/".length)));
				} else if (href.startsWith("zealot://type/")) {
					getNavigator().openType(decodeURIComponent(href.slice("zealot://type/".length)));
				} else {
					window.open(href, "_blank", "noopener,noreferrer");
				}
				return true;
			},
			dispatchTransaction: (tr: Transaction) => {
				if (!this._view) return;
				const nextState = this._view.state.apply(tr);
				this._view.updateState(nextState);

				const trigger = detectWikilinkTrigger(nextState);
				if (trigger) {
					if (this._wikilinkPickerEl) {
						this._updateWikilinkPicker(trigger.from, trigger.query);
					} else {
						this._showWikilinkPicker(trigger.from, trigger.query);
					}
				} else if (this._wikilinkPickerEl) {
					this._hideWikilinkPicker();
				}

				const emojiTrigger = detectEmojiTrigger(nextState);
				if (emojiTrigger) {
					if (this._emojiPickerEl) {
						this._emojiPickerFrom = emojiTrigger.from;
						this._updateEmojiPickerContent(emojiTrigger.from, emojiTrigger.query);
					} else {
						this._showEmojiPicker(emojiTrigger.from, emojiTrigger.query);
					}
				} else if (this._emojiPickerEl) {
					this._hideEmojiPicker();
				}

				if (!tr.docChanged) return;
				if (this._saveTimer !== null) window.clearTimeout(this._saveTimer);
				this._saveTimer = window.setTimeout(() => {
					if (!this._view) return;
					const value = serializeZealotScript(this._view.state.doc);
					if (value === this._lastValue) return;
					this._lastValue = value;
					this.dispatchEvent(new CustomEvent("change", { detail: value, bubbles: true }));
				}, 500);
			},
		});
	}

	disconnectedCallback() {
		if (this._saveTimer !== null) window.clearTimeout(this._saveTimer);
		this._hideWikilinkPicker();
		this._hideEmojiPicker();
		this._view?.destroy();
		this._view = null;
	}

	get content(): string {
		if (!this._view) return this._lastValue;
		return serializeZealotScript(this._view.state.doc);
	}

	set content(value: string) {
		this._lastValue = value;
		if (!this._view) return;
		const doc = parseZealotScript(ZealotSchema, value);
		const tr = this._view.state.tr.replaceWith(0, this._view.state.doc.content.size, doc.content);
		this._view.dispatch(tr);
	}

	focus(): void {
		this._view?.focus();
	}
}

customElements.define("zealotscript-editor", ZealotScriptEditor);
