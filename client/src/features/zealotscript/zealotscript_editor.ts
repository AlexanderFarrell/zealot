import { EditorState, TextSelection, Transaction } from "prosemirror-state";
import { EditorView } from "prosemirror-view";
import { exampleSetup } from "prosemirror-example-setup";
import ZealotSchema from "./schema";
import { parseZealotScript } from "./parser";
import { serializeZealotScript } from "./serializer";
import { router } from "../router/router";
import API from "../../api/api";
import { liftListItem, sinkListItem } from "prosemirror-schema-list";
import {keymap} from "prosemirror-keymap"
import {InputRule, inputRules} from "prosemirror-inputrules";
import { MarkType, Schema } from "prosemirror-model";

export type ZealotEditorOptions = {
	content?: string;
	onUpdate?: (value: string, view: EditorView) => void;
	debounceMs?: number;
	handleTab?: boolean;
}

const isZealotDebugEnabled = () => {
	if (typeof window === "undefined") return false;
	return window.localStorage.getItem("zealotscript_debug") === "1";
}

const debugZealot = (...args: unknown[]) => {
	if (!isZealotDebugEnabled()) return;
	console.debug("[ZealotScript]", ...args);
}

const buildInputRules = (schema: Schema) => {
	const rules: InputRule[] = [];

	const addMarkRule = (markType: MarkType | undefined, regex: RegExp) => {
		if (!markType) return;
		rules.push(new InputRule(regex, (state, match, start, end) => {
			const inner = match[1];
			if (!inner || typeof inner !== "string") return null;
			const node = schema.text(inner, [markType.create()]);
			return state.tr.replaceWith(start, end, node);
		}));
	}

	addMarkRule(schema.marks.strong, /\*\*([^*\n]+)\*\*$/);
	addMarkRule(schema.marks.em, /(?<!\*)\*([^*\n]+)\*$/);
	addMarkRule(schema.marks.strike, /~~([^~\n]+)~~$/);
	addMarkRule(schema.marks.code, /`([^`\n]+)`$/);
	addMarkRule(schema.marks.underline, /(?<![A-Za-z0-9_])_([^_\n]+)_$/);
	addMarkRule(schema.marks.highlight, /<mark>([^<\n]+)<\/mark>$/i);
	addMarkRule(schema.marks.subscript, /<sub>([^<\n]+)<\/sub>$/i);
	addMarkRule(schema.marks.superscript, /<sup>([^<\n]+)<\/sup>$/i);

	// [[Page Name]]
	rules.push(new InputRule(/\[\[([^\]]+)\]\]$/, (state, match, start, end) => {
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

	rules.push(new InputRule(/:::link\|([^|\n]*)\|([^|\n]+)$/, (state, match, start, end) => {
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

const moveToLineBoundary = (
	state: EditorState,
	dispatch: ((tr: Transaction) => void) | undefined,
	toEnd: boolean,
	extendSelection: boolean
) => {
	const { $head, $anchor } = state.selection;
	if (!$head.parent.isTextblock) return false;

	const lineStart = $head.start();
	const lineEnd = lineStart + $head.parent.content.size;
	const target = toEnd ? lineEnd : lineStart;

	let selection: TextSelection;
	if (extendSelection) {
		selection = TextSelection.create(state.doc, $anchor.pos, target);
	} else {
		selection = TextSelection.create(state.doc, target, target);
	}

	if (dispatch) {
		dispatch(state.tr.setSelection(selection));
	}
	return true;
}

export const createZealotEditorState = (content: string) => {
	debugZealot("create state with content", content);
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

const SCREENSHOT_FOLDER = "screenshots";

const getImageExtension = (file: File): string => {
	if (file.name && file.name.includes(".")) {
		return file.name.slice(file.name.lastIndexOf(".") + 1).toLowerCase();
	}
	const type = file.type.toLowerCase();
	if (type.endsWith("/png")) return "png";
	if (type.endsWith("/jpeg")) return "jpg";
	if (type.endsWith("/jpg")) return "jpg";
	if (type.endsWith("/gif")) return "gif";
	if (type.endsWith("/webp")) return "webp";
	if (type.endsWith("/svg+xml")) return "svg";
	return "png";
}

const buildScreenshotName = (file: File) => {
	const stamp = new Date().toISOString().replace(/[:.]/g, "-");
	const ext = getImageExtension(file);
	return `screenshot-${stamp}.${ext}`;
}

const handleImagePaste = (view: EditorView, event: ClipboardEvent) => {
	const items = event.clipboardData?.items;
	if (!items || items.length === 0) return false;

	const imageItem = Array.from(items).find((item) => item.type.startsWith("image/"));
	if (!imageItem) return false;

	const rawFile = imageItem.getAsFile();
	if (!rawFile) return false;

	const filename = buildScreenshotName(rawFile);
	const uploadFile = new File([rawFile], filename, {type: rawFile.type});
	const bookmark = view.state.selection.getBookmark();

	event.preventDefault();

	API.media.upload_file(uploadFile, SCREENSHOT_FOLDER)
		.then(() => {
			const src = `/api/media/${SCREENSHOT_FOLDER}/${filename}`;
			const imageNode = view.state.schema.nodes.image;
			if (!imageNode) return;
			const selection = bookmark.resolve(view.state.doc);
			const tr = view.state.tr.replaceRangeWith(selection.from, selection.to, imageNode.create({
				src,
				alt: filename
			}));
			view.dispatch(tr);
		})
		.catch((error) => {
			console.error("Failed to upload pasted image", error);
		});

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
				router.navigate(`/item/${title}`);
				return true;
			},
			handleClick: (view, pos, event) => {
				const target = event.target as HTMLElement | null;
				const anchor = target?.closest("a[href]") as HTMLAnchorElement | null;
				if (!anchor) return false;
				if (anchor.hasAttribute("data-itemlink")) return false;

				const href = anchor.getAttribute("href") || "";
				if (href.trim().length === 0) return false;

				event.preventDefault();
				event.stopPropagation();

				if (href.startsWith("/")) {
					router.navigate(href);
				} else {
					window.open(href, "_blank", "noopener,noreferrer");
				}
				return true;
			},
			handleKeyDown: (view, event) => {
				if (event.key === "Home") {
					event.preventDefault();
					return moveToLineBoundary(view.state, view.dispatch, false, event.shiftKey);
				}

				if (event.key === "End") {
					event.preventDefault();
					return moveToLineBoundary(view.state, view.dispatch, true, event.shiftKey);
				}

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
		handlePaste: (view, event) => {
			return handleImagePaste(view, event as ClipboardEvent);
		},
		dispatchTransaction: (tr) => {
			const nextState = view.state.apply(tr);
			view.updateState(nextState);

			if (!tr.docChanged || !options.onUpdate) return;
			if (saveTimer !== null) window.clearTimeout(saveTimer);

				const delay = options.debounceMs ?? 500;
				saveTimer = window.setTimeout(() => {
					const value = serializeZealotScript(view.state.doc);
					debugZealot("serialized update value", value);
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
