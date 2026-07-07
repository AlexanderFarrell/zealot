import { EditorState, TextSelection, type Transaction } from "prosemirror-state";
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
import { tableEditing, columnResizing, goToNextCell, addRowBefore, addRowAfter, deleteRow, addColumnBefore, addColumnAfter, deleteColumn, deleteTable, toggleHeaderRow } from "prosemirror-tables";
import { insertTable, insertAdmonition, insertYoutubeEmbed, insertMermaidBlock, extractYouTubeVideoId, insertDetails, insertSpoiler, insertColumns, insertTabs, isInTable } from "./commands";
import { TabsView } from "./zealotscript_view";
import { lookupEmoji } from "./emoji_map";
import { hasIcon, setIconRefElement } from "./icon_registry";
import { ShortcodePicker, detectShortcodeTriggerInText, type ShortcodeSuggestion } from "./shortcode_picker";
import { codeHighlightPlugin } from "./code_highlight_plugin";
import { getNavigator, commands, ModalCommands } from "@websoil/engine";
import { MediaAPI } from "@zealot/api/src/media";
import { ItemSearchInline } from "../views/item_search_inline";
import type { Item } from "@zealot/domain/src/item";
import type MermaidType from "mermaid";

const mediaApi = new MediaAPI('/api');

type PMCommand = (state: EditorState, dispatch?: (tr: Transaction) => void) => boolean;

let _mermaidLib: typeof MermaidType | null = null;
const loadMermaidLib = async (): Promise<typeof MermaidType> => {
	if (_mermaidLib) return _mermaidLib;
	const mod = await import("mermaid");
	_mermaidLib = mod.default;
	_mermaidLib.initialize({ startOnLoad: false, theme: "neutral" });
	return _mermaidLib;
};

let _mermaidCounter = 0;

export class MermaidBlockView {
	dom: HTMLElement;
	contentDOM: HTMLElement;
	private _preview: HTMLDivElement;
	private _renderTimer: ReturnType<typeof setTimeout> | null = null;

	constructor(node: import("prosemirror-model").Node) {
		const wrapper = document.createElement("div");
		wrapper.className = "zealot-mermaid-editor-block";

		const pre = document.createElement("pre");
		pre.className = "zealot-mermaid-source";
		wrapper.appendChild(pre);

		const preview = document.createElement("div");
		preview.className = "zealot-mermaid-preview";
		preview.contentEditable = "false";
		wrapper.appendChild(preview);

		this.dom = wrapper;
		this.contentDOM = pre;
		this._preview = preview;

		this._scheduleRender(node.textContent);
	}

	update(node: import("prosemirror-model").Node): boolean {
		if (node.type.name !== "code_block" || node.attrs.language !== "mermaid") return false;
		this._scheduleRender(node.textContent);
		return true;
	}

	private _scheduleRender(source: string): void {
		if (this._renderTimer !== null) clearTimeout(this._renderTimer);
		this._renderTimer = setTimeout(() => void this._render(source), 300);
	}

	private async _render(source: string): Promise<void> {
		if (!source.trim()) {
			this._preview.innerHTML = "";
			return;
		}
		try {
			const mermaid = await loadMermaidLib();
			const id = `zealot-mermaid-editor-${++_mermaidCounter}`;
			const { svg } = await mermaid.render(id, source);
			this._preview.className = "zealot-mermaid-preview zealot-mermaid";
			this._preview.innerHTML = svg;
		} catch (err) {
			this._preview.className = "zealot-mermaid-preview zealot-mermaid-error";
			this._preview.textContent = err instanceof Error ? err.message : String(err);
		}
	}

	destroy(): void {
		if (this._renderTimer !== null) clearTimeout(this._renderTimer);
	}
}

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

export class IconRefView {
	dom: HTMLElement;

	constructor(node: import("prosemirror-model").Node) {
		this.dom = document.createElement("span");
		this.dom.className = "zealot-icon";
		this.dom.contentEditable = "false";
		this._render(node);
	}

	_render(node: import("prosemirror-model").Node): void {
		const name = node.attrs.name as string;
		setIconRefElement(this.dom, name, { className: "zealot-icon", title: name });
	}

	update(node: import("prosemirror-model").Node): boolean {
		if (node.type.name !== "icon_ref") return false;
		this._render(node);
		return true;
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
			const marked = schema.text(inner, [markType.create()]);
			const space = schema.text(" ");
			const tr = state.tr.replaceWith(start, end, marked);
			const spacePos = tr.mapping.map(start) + marked.nodeSize;
			return tr.replaceWith(spacePos, spacePos, space);
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
		if (emoji) return state.tr.replaceWith(start, end, schema.text(emoji));
		if (hasIcon(shortcode)) {
			const iconRefType = schema.nodes["icon_ref"];
			if (iconRefType) {
				return state.tr.replaceWith(start, end, iconRefType.createChecked({ name: shortcode }));
			}
		}
		return null;
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

const tableTab = (state: EditorState, dispatch?: (tr: Transaction) => void, view?: EditorView): boolean => {
	if (!isInTable(state)) return false;
	if (goToNextCell(1)(state, dispatch, view)) return true;
	// Last cell — append a new row and move into its first cell.
	const { schema } = state;
	const rowType = schema.nodes["table_row"];
	const cellType = schema.nodes["table_cell"];
	const paragraph = schema.nodes["paragraph"];
	if (!rowType || !cellType || !paragraph) return false;
	const { $from } = state.selection;
	let rowDepth = -1;
	for (let d = $from.depth; d >= 0; d--) {
		if ($from.node(d).type === rowType) { rowDepth = d; break; }
	}
	if (rowDepth < 0) return false;
	const colCount = $from.node(rowDepth).childCount;
	const newCells = Array.from({ length: colCount }, () => cellType.createAndFill({}, [paragraph.create()])!);
	const newRow = rowType.create({}, newCells);
	const rowEnd = $from.after(rowDepth);
	if (dispatch) {
		const tr = state.tr.insert(rowEnd, newRow);
		const firstCellPos = rowEnd + 2;
		tr.setSelection(TextSelection.near(tr.doc.resolve(firstCellPos)));
		dispatch(tr.scrollIntoView());
	}
	return true;
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
		columnResizing(),
		tableEditing(),
		keymap({ Tab: tableTab, "Shift-Tab": goToNextCell(-1) }),
		keymap(buildKeymap(schema)),
		keymap(baseKeymap),
		dropCursor(),
		gapCursor(),
		history(),
		keymap({ "Mod-k": addLink, "Mod-Shift-k": removeLink }),
		buildInputRules(schema),
		codeHighlightPlugin(),
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

function detectShortcodeTrigger(state: EditorState): { from: number; query: string } | null {
	const { $from } = state.selection;
	if ($from.depth === 0) return null;
	const text = $from.parent.textBetween(0, $from.parentOffset, null, "\0");
	const trigger = detectShortcodeTriggerInText(text);
	if (!trigger) return null;
	return { from: $from.start() + trigger.from, query: trigger.query };
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
	private _shortcodePicker: ShortcodePicker | null = null;
	private _shortcodePickerFrom: number | null = null;
	private _toolbar: (HTMLElement & { _syncTableButtons?: () => void }) | null = null;
	private _toolbarMenuCleanups: Array<() => void> = [];

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
		const insertText = `[[${item.Title}]] `;
		const textNode = state.schema.text(insertText);
		let tr = state.tr.replaceWith(from, to, textNode);
		const cursorPos = from + insertText.length;
		tr = tr.setSelection(TextSelection.create(tr.doc, cursorPos));
		this._view.dispatch(tr);
		this._hideWikilinkPicker();
		this._view.focus();
	}

	private _getShortcodePicker(): ShortcodePicker {
		if (!this._shortcodePicker) {
			this._shortcodePicker = new ShortcodePicker({
				containsTarget: (target) => this.contains(target),
				onSelect: (suggestion) => this._insertShortcodeSuggestion(suggestion),
			});
		}
		return this._shortcodePicker;
	}

	private _updateShortcodePicker(from: number, query: string): void {
		if (!this._view) return;
		this._shortcodePickerFrom = from;
		const coords = this._view.coordsAtPos(from);
		this._getShortcodePicker().update({ left: coords.left, bottom: coords.bottom }, query);
	}

	private _insertShortcodeSuggestion(suggestion: ShortcodeSuggestion): void {
		if (suggestion.kind === "emoji") {
			this._insertEmoji(suggestion.emoji);
			return;
		}
		this._insertIcon(suggestion.shortcode);
	}

	private _insertEmoji(emoji: string): void {
		if (!this._view || this._shortcodePickerFrom === null) return;
		const { state } = this._view;
		const from = this._shortcodePickerFrom;
		const to = state.selection.from;
		const tr = state.tr.replaceWith(from, to, state.schema.text(emoji));
		this._view.dispatch(tr);
		this._hideShortcodePicker();
		this._view.focus();
	}

	private _insertIcon(name: string): void {
		if (!this._view || this._shortcodePickerFrom === null) return;
		const { state } = this._view;
		const from = this._shortcodePickerFrom;
		const to = state.selection.from;
		const iconRefType = state.schema.nodes["icon_ref"];
		const replacement = iconRefType
			? iconRefType.createChecked({ name })
			: state.schema.text(`:${name}:`);
		const tr = state.tr.replaceWith(from, to, replacement);
		this._view.dispatch(tr);
		this._hideShortcodePicker();
		this._view.focus();
	}

	private _hideShortcodePicker(): void {
		this._shortcodePicker?.hide();
		this._shortcodePickerFrom = null;
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

		interface DropdownEntry {
			label: string;
			title: string;
			onClick: () => void;
		}

		const dropdown = (label: string, title: string, entries: DropdownEntry[]): HTMLButtonElement => {
			const trigger = document.createElement("button");
			trigger.type = "button";
			trigger.textContent = `${label} ▾`;
			trigger.title = title;
			trigger.className = "zealotscript-toolbar-dropdown-trigger";

			let panel: HTMLDivElement | null = null;

			const close = () => {
				panel?.remove();
				panel = null;
				document.removeEventListener("mousedown", onOutsideMousedown, true);
				document.removeEventListener("keydown", onKeydown, true);
			};

			const onOutsideMousedown = (e: MouseEvent) => {
				if (panel && e.target instanceof Node && !panel.contains(e.target) && e.target !== trigger) {
					close();
				}
			};

			const onKeydown = (e: KeyboardEvent) => {
				if (e.key === "Escape") close();
			};

			const open = () => {
				panel = document.createElement("div");
				panel.className = "zealotscript-toolbar-dropdown-panel";
				panel.style.display = "block";

				for (const entry of entries) {
					const item = document.createElement("button");
					item.type = "button";
					item.className = "ctx-menu-item";
					item.textContent = entry.label;
					item.title = entry.title;
					item.addEventListener("mousedown", (e) => {
						e.preventDefault();
						close();
						entry.onClick();
					});
					panel.appendChild(item);
				}

				document.body.appendChild(panel);
				const rect = trigger.getBoundingClientRect();
				const panelRect = panel.getBoundingClientRect();
				const vw = window.innerWidth;
				panel.style.left = `${Math.min(rect.left, vw - panelRect.width - 8)}px`;
				panel.style.top = `${rect.bottom + 4}px`;

				document.addEventListener("mousedown", onOutsideMousedown, true);
				document.addEventListener("keydown", onKeydown, true);
			};

			trigger.addEventListener("mousedown", (e) => {
				e.preventDefault();
				if (panel) {
					close();
				} else {
					open();
				}
			});

			this._toolbarMenuCleanups.push(close);
			return trigger;
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

		const insertTaskList = () => {
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
		};

		bar.appendChild(btn("H1", "Heading 1", () => setHeading(1)));
		bar.appendChild(btn("H2", "Heading 2", () => setHeading(2)));
		bar.appendChild(btn("H3", "Heading 3", () => setHeading(3)));

		const sep1 = document.createElement("span");
		sep1.className = "zealotscript-toolbar-sep";
		bar.appendChild(sep1);

		bar.appendChild(btn("B", "Bold (Ctrl+B)", () => toggleMark("strong")));
		bar.appendChild(btn("I", "Italic (Ctrl+I)", () => toggleMark("em")));

		bar.appendChild(dropdown("Format", "More formatting", [
			{ label: "Strikethrough", title: "Strikethrough", onClick: () => toggleMark("strike") },
			{ label: "Underline", title: "Underline", onClick: () => toggleMark("underline") },
			{ label: "Inline Code", title: "Inline Code", onClick: () => toggleMark("code") },
			{ label: "Task List", title: "Insert Task List", onClick: insertTaskList },
		]));

		bar.appendChild(dropdown("Insert", "Insert block", [
			{ label: "Table", title: "Insert Table", onClick: () => runCommand(insertTable) },
			{ label: "Note", title: "Insert Note", onClick: () => runCommand(insertAdmonition("note")) },
			{ label: "Warning", title: "Insert Warning", onClick: () => runCommand(insertAdmonition("warning")) },
			{ label: "Tip", title: "Insert Tip", onClick: () => runCommand(insertAdmonition("tip")) },
			{
				label: "YouTube Video", title: "Insert YouTube Video", onClick: () => {
					const input = window.prompt("YouTube URL or video ID:");
					if (!input) return;
					const videoId = extractYouTubeVideoId(input.trim());
					if (videoId) runCommand(insertYoutubeEmbed(videoId));
				},
			},
			{ label: "Mermaid Diagram", title: "Insert Mermaid Diagram", onClick: () => runCommand(insertMermaidBlock) },
			{
				label: "Details Block", title: "Insert Details block", onClick: () => {
					const summary = window.prompt("Summary text:", "Details") || "Details";
					runCommand(insertDetails(summary));
				},
			},
			{ label: "Spoiler Block", title: "Insert Spoiler block", onClick: () => runCommand(insertSpoiler) },
			{ label: "2-Column Layout", title: "Insert 2-column layout", onClick: () => runCommand(insertColumns(2)) },
			{ label: "Tabs", title: "Insert Tabs", onClick: () => runCommand(insertTabs(["Tab 1", "Tab 2"])) },
		]));

		const tableEditSep = document.createElement("span");
		tableEditSep.className = "zealotscript-toolbar-sep zealotscript-toolbar-table-edit";
		bar.appendChild(tableEditSep);

		const tableDropdown = dropdown("Table", "Table editing", [
			{ label: "Add Row Above", title: "Add Row Above", onClick: () => runCommand(addRowBefore) },
			{ label: "Add Row Below", title: "Add Row Below", onClick: () => runCommand(addRowAfter) },
			{ label: "Delete Row", title: "Delete Row", onClick: () => runCommand(deleteRow) },
			{ label: "Add Column Before", title: "Add Column Before", onClick: () => runCommand(addColumnBefore) },
			{ label: "Add Column After", title: "Add Column After", onClick: () => runCommand(addColumnAfter) },
			{ label: "Delete Column", title: "Delete Column", onClick: () => runCommand(deleteColumn) },
			{ label: "Toggle Header Row", title: "Toggle Header Row", onClick: () => runCommand(toggleHeaderRow) },
			{ label: "Delete Table", title: "Delete Table", onClick: () => runCommand(deleteTable) },
		]);
		tableDropdown.className += " zealotscript-toolbar-table-edit";
		bar.appendChild(tableDropdown);

		// Show/hide table editing controls based on cursor position.
		const tableEditEls = bar.querySelectorAll<HTMLElement>(".zealotscript-toolbar-table-edit");
		const syncTableButtons = () => {
			if (!this._view) return;
			const inTable = isInTable(this._view.state);
			tableEditEls.forEach((el) => { el.style.display = inTable ? "" : "none"; });
		};
		syncTableButtons();
		// Will be refreshed on every transaction via dispatchTransaction.
		(bar as HTMLElement & { _syncTableButtons?: () => void })._syncTableButtons = syncTableButtons;

		return bar;
	}

	connectedCallback() {
		if (this._view) return;

		const toolbar = this._buildToolbar();
		this._toolbar = toolbar;
		this.appendChild(toolbar);

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
				icon_ref: (node) => new IconRefView(node),
				list_item: (node, view, getPos) => new TaskListItemView(node, view, getPos),
				tabs: (node) => new TabsView(node),
				code_block: (node) => {
					if (node.attrs.language === "mermaid") return new MermaidBlockView(node);
					const pre = document.createElement("pre");
					pre.setAttribute("data-language", (node.attrs.language as string) ?? "");
					const code = document.createElement("code");
					pre.appendChild(code);
					return { dom: pre, contentDOM: code };
				},
			},
			handleKeyDown: (_view, event) => {
				if (event.key === "Escape" && (this._wikilinkPickerEl || this._shortcodePicker?.isOpen)) {
					event.preventDefault();
					this._hideWikilinkPicker();
					this._hideShortcodePicker();
					return true;
				}
				if (this._wikilinkPickerEl && (event.key === "ArrowDown" || event.key === "ArrowUp" || event.key === "Enter")) {
					event.preventDefault();
					this._wikilinkPickerSearch?.forwardKeyEvent(event);
					return true;
				}
				const isMac = navigator.platform.toUpperCase().includes("MAC");
				const mod = isMac ? event.metaKey : event.ctrlKey;
				if (mod && !event.shiftKey && !event.altKey) {
					if (event.key === "o") { event.preventDefault(); commands.runner.run(ModalCommands.openGlobalSearch); return true; }
					if (event.key === "n") { event.preventDefault(); commands.runner.run(ModalCommands.newItem); return true; }
					if (event.key === "p") { event.preventDefault(); commands.runner.run(ModalCommands.openCommandRunner); return true; }
				}
				if (event.key !== "Tab") return false;
				event.preventDefault();
				// Table Tab/Shift-Tab: handled first.
				if (isInTable(_view.state)) {
					const cmd = event.shiftKey ? goToNextCell(-1) : tableTab;
					if (cmd(_view.state, _view.dispatch, _view)) return true;
				}
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
			handlePaste: (_view, event) => {
				// Inside a code block, paste raw text verbatim: ProseMirror's default
				// clipboard parser turns pasted line breaks into extra hard breaks /
				// block splits, which doubles the newlines. Insert the plain-text
				// clipboard payload as a single text node (with `\n` preserved) instead.
				if (_view.state.selection.$from.parent.type.spec.code) {
					const text = event.clipboardData?.getData("text/plain") ?? "";
					if (text.length === 0) return false;
					event.preventDefault();
					const normalized = text.replace(/\r\n?/g, "\n");
					_view.dispatch(_view.state.tr.insertText(normalized).scrollIntoView());
					return true;
				}
				const items = Array.from(event.clipboardData?.items ?? []);
				const imageItem = items.find((i) => i.type.startsWith("image/"));
				if (!imageItem) return false;
				event.preventDefault();
				const file = imageItem.getAsFile();
				if (!file) return false;
				const ext = file.type.split("/")[1] ?? "png";
				const filename = `paste-${Date.now()}.${ext}`;
				const namedFile = new File([file], filename, { type: file.type });
				void mediaApi.UploadFolder(namedFile, "paste").then(() => {
					if (!this._view) return;
					const url = `/api/media/paste/${filename}`;
					const { state } = this._view;
					const imageNodeType = state.schema.nodes["image"];
					if (!imageNodeType) return;
					const imageNode = imageNodeType.create({ src: url, alt: filename });
					const tr = state.tr.replaceSelectionWith(imageNode, false);
					this._view.dispatch(tr);
				});
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

				const shortcodeTrigger = detectShortcodeTrigger(nextState);
				if (shortcodeTrigger) {
					this._updateShortcodePicker(shortcodeTrigger.from, shortcodeTrigger.query);
				} else if (this._shortcodePicker?.isOpen) {
					this._hideShortcodePicker();
				}

				this._toolbar?._syncTableButtons?.();

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
		this._hideShortcodePicker();
		for (const close of this._toolbarMenuCleanups) close();
		this._toolbarMenuCleanups = [];
		this._view?.destroy();
		this._view = null;
		this._toolbar = null;
		this.innerHTML = '';
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
