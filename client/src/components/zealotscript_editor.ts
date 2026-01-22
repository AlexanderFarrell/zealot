import { EditorState, Transaction } from "prosemirror-state";
import { EditorView } from "prosemirror-view";
import { exampleSetup } from "prosemirror-example-setup";
import ZealotSchema from "../core/zealotscript/schema";
import { parseZealotScript } from "../core/zealotscript/parser";
import { serializeZealotScript } from "../core/zealotscript/serializer";
import { liftListItem, sinkListItem } from "prosemirror-schema-list";
import {keymap} from "prosemirror-keymap"
import { router } from "../core/router";
import {InputRule, inputRules, textblockTypeInputRule} from "prosemirror-inputrules";
import { MarkType, Schema } from "prosemirror-model";

export type ZealotEditorOptions = {
	content?: string;
	onUpdate?: (value: string, view: EditorView) => void;
	debounceMs?: number;
	handleTab?: boolean;
}

const buildInputRules = (schema: Schema) => {
	const rules = [];

	// [[Page Name]]
	rules.push(new InputRule(/\[\[([^\]]+)\]\]/g, (state, match, start, end) => {
		const title = match[1].trim();
		const href = `/item/${title}`;
		const node = schema.nodes.itemlink.create({title, href});
		const tr = state.tr.replaceWith(start, end, node);
		return tr.insertText(" ", start + 1); // Add space at the end
	}))

	// [Label](url)
	const linkMark = schema.marks.link;
	if (linkMark) {
		rules.push(new InputRule(/\[([^\]]+)\]\(([^)]+)\)\s$/, (state, match, start, end) => {
			const label = match[1];
			const href = match[2];
			const mark = schema.marks.link?.create({href});
			const text = mark ? schema.text(label, [mark]) : schema.text(label);

			const tr = state.tr.replaceWith(start, end, text);

			const insertPos = tr.mapping.map(start) + text.nodeSize;
			return tr.insertText(" ", insertPos);
		}));
	};

	rules.push(new InputRule(/:::link\|([^|\n]*)\|([^|\n]+)/g, (state, match, start, end) => {
		const label = match[1].trim();
		const href = match[2].trim();
		const text = label.length > 0 ? label : href;
		const mark = schema.marks.link?.create({href});
		const node = mark ? schema.text(text, [mark]) : schema.text(text);
		const tr = state.tr.replaceWith(start, end, node);
		return tr;
	}))

	return inputRules({rules});
}

const isInList = (state: EditorState) => {
	const listItemType = state.schema.nodes.list_item;
	if (!listItemType) return false;

	const {$from} = state.selection;
	for (let depth = $from.depth; depth > 0; depth--) {
		if ($from.node(depth).type == listItemType) return true;
	}
	return false;
}

export const createZealotEditorState = (content: string) => {
	return EditorState.create({
		schema: ZealotSchema,
		doc: parseZealotScript(ZealotSchema, content),
		plugins: [
			...exampleSetup({ schema: ZealotSchema }),
			keymap({
				"Mod-k": addLink,
				"Mod-Shift-k": removeLink
			}),
			buildInputRules(ZealotSchema)
		]
	})
}

const linkMark = ZealotSchema.marks.link;

const addLink = (state: EditorState, dispatch?: (tr: Transaction) => void) => {
	const {from, to} = state.selection;
	const href = window.prompt("Add Link URL");
	if (!href) return false;
	if (dispatch) {
		dispatch(state.tr.addMark(from, to, linkMark.create({href})));
	}
	return true;
}

const removeLink = (state: EditorState, dispatch?: (tr: Transaction) => void) => {
	const {from, to} = state.selection;
	if (dispatch) {
		dispatch(state.tr.removeMark(from, to, linkMark));
	}
	return true;
}

export const createZealotEditorView = (
	host: HTMLElement,
	options: ZealotEditorOptions = {}
) => {
	const initial = options.content ?? "";
	const state = createZealotEditorState(initial);
	let saveTimer: number | null = null;
	let lastValue = initial;

	const view = new EditorView(host, {
		state,
		handleClickOn: (view, pos, node, nodePos, event) => {
			if (node.type.name !== "itemlink") return false;

			event.preventDefault();
			event.stopPropagation();

			const title = node.attrs.title || "";
			const href = node.attrs.href || "";

			router.navigate(`/item/${title}`);
		},
		handleKeyDown: (view, event) => {
			if (!options.handleTab) return false;
			if (event.key !== "Tab") return false;
			event.preventDefault();

			const listItemType = view.state.schema.nodes.list_item;
			if (listItemType && isInList(view.state)) {
				const command = event.shiftKey 
					? liftListItem(listItemType)
					: sinkListItem(listItemType);
				if (command(view.state, view.dispatch)) return true;
			}

			view.dispatch(view.state.tr.insertText("\t"));
			return true;
		},
		dispatchTransaction: (tr) => {
			const nextState = view.state.apply(tr);
			view.updateState(nextState);

			if (!tr.docChanged || !options.onUpdate) return;
			if (saveTimer !== null) window.clearTimeout(saveTimer);

			const delay = options.debounceMs ?? 500;
			saveTimer = window.setTimeout(() => {
				const value = serializeZealotScript(view.state.doc);
				if (value === lastValue) return;
				lastValue = value;
				options.onUpdate?.(value, view);
			}, delay);
		}
	});

	view.dom.setAttribute("name", "editor_prose");
	return view;
}

export default createZealotEditorView;