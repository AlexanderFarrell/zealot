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
import { insertTable, insertAdmonition } from "./commands";

type PMCommand = (state: EditorState, dispatch?: (tr: Transaction) => void) => boolean;

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

export class ZealotScriptEditor extends HTMLElement {
	private _view: EditorView | null = null;
	private _saveTimer: number | null = null;
	private _lastValue = "";

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

		bar.appendChild(btn("Table", "Insert Table", () => runCommand(insertTable)));
		bar.appendChild(btn("Note", "Insert Note", () => runCommand(insertAdmonition("note"))));
		bar.appendChild(btn("Warning", "Insert Warning", () => runCommand(insertAdmonition("warning"))));
		bar.appendChild(btn("Tip", "Insert Tip", () => runCommand(insertAdmonition("tip"))));

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
			handleKeyDown: (_view, event) => {
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
				const target = event.target as HTMLElement | null;
				const anchor = target?.closest("a[href]") as HTMLAnchorElement | null;
				if (!anchor) return false;
				const href = anchor.getAttribute("href") || "";
				if (href.trim().length === 0) return false;
				event.preventDefault();
				event.stopPropagation();
				window.open(href, "_blank", "noopener,noreferrer");
				return true;
			},
			dispatchTransaction: (tr: Transaction) => {
				if (!this._view) return;
				const nextState = this._view.state.apply(tr);
				this._view.updateState(nextState);

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
}

customElements.define("zealotscript-editor", ZealotScriptEditor);
