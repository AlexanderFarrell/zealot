import { EditorState, TextSelection, Transaction } from "prosemirror-state";
import { EditorView } from "prosemirror-view";
import type { NodeView } from "prosemirror-view";
import { buildKeymap } from "prosemirror-example-setup";
import ZealotSchema from "./schema";
import { parseZealotScript } from "./parser";
import { serializeZealotScript } from "./serializer";
import { router } from "../router/router";
import API from "../../api/api";
import { liftListItem, sinkListItem } from "prosemirror-schema-list";
import {keymap} from "prosemirror-keymap"
import {InputRule, inputRules, textblockTypeInputRule} from "prosemirror-inputrules";
import { MarkType, Node as PMNode, Schema } from "prosemirror-model";
import { baseKeymap } from "prosemirror-commands";
import { dropCursor } from "prosemirror-dropcursor";
import { gapCursor } from "prosemirror-gapcursor";
import { history } from "prosemirror-history";
import hljs from "highlight.js/lib/core";
import bash from "highlight.js/lib/languages/bash";
import c from "highlight.js/lib/languages/c";
import cpp from "highlight.js/lib/languages/cpp";
import css from "highlight.js/lib/languages/css";
import go from "highlight.js/lib/languages/go";
import ini from "highlight.js/lib/languages/ini";
import javascript from "highlight.js/lib/languages/javascript";
import json from "highlight.js/lib/languages/json";
import markdown from "highlight.js/lib/languages/markdown";
import python from "highlight.js/lib/languages/python";
import rust from "highlight.js/lib/languages/rust";
import sql from "highlight.js/lib/languages/sql";
import typescript from "highlight.js/lib/languages/typescript";
import xml from "highlight.js/lib/languages/xml";
import yaml from "highlight.js/lib/languages/yaml";
import Popups from "../../shared/popups";
import type { Item } from "../../api/item";

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

hljs.registerLanguage("bash", bash);
hljs.registerLanguage("sh", bash);
hljs.registerLanguage("shell", bash);
hljs.registerLanguage("zsh", bash);
hljs.registerLanguage("c", c);
hljs.registerLanguage("cpp", cpp);
hljs.registerLanguage("c++", cpp);
hljs.registerLanguage("css", css);
hljs.registerLanguage("go", go);
hljs.registerLanguage("ini", ini);
hljs.registerLanguage("toml", ini);
hljs.registerLanguage("javascript", javascript);
hljs.registerLanguage("js", javascript);
hljs.registerLanguage("json", json);
hljs.registerLanguage("markdown", markdown);
hljs.registerLanguage("md", markdown);
hljs.registerLanguage("python", python);
hljs.registerLanguage("py", python);
hljs.registerLanguage("rust", rust);
hljs.registerLanguage("rs", rust);
hljs.registerLanguage("sql", sql);
hljs.registerLanguage("typescript", typescript);
hljs.registerLanguage("ts", typescript);
hljs.registerLanguage("html", xml);
hljs.registerLanguage("xml", xml);
hljs.registerLanguage("yaml", yaml);
hljs.registerLanguage("yml", yaml);
hljs.registerLanguage("zig", cpp);

const CODE_LANGUAGE_OPTIONS: Array<{ value: string; label: string }> = [
	{ value: "", label: "Plain text" },
	{ value: "ts", label: "TypeScript (ts)" },
	{ value: "js", label: "JavaScript (js)" },
	{ value: "json", label: "JSON" },
	{ value: "html", label: "HTML" },
	{ value: "css", label: "CSS" },
	{ value: "md", label: "Markdown (md)" },
	{ value: "py", label: "Python (py)" },
	{ value: "go", label: "Go" },
	{ value: "sql", label: "SQL" },
	{ value: "yaml", label: "YAML" },
	{ value: "toml", label: "TOML" },
	{ value: "bash", label: "Bash" },
	{ value: "zsh", label: "Zsh" },
	{ value: "c", label: "C" },
	{ value: "cpp", label: "C++ (cpp)" },
	{ value: "rust", label: "Rust" },
	{ value: "zig", label: "Zig" }
];

const buildInputRules = (schema: Schema) => {
	const rules: InputRule[] = [];
	const codeBlockType = schema.nodes.code_block;

	if (codeBlockType) {
		// Convert when user types a trailing space so language can be entered first: ```ts<space>
		rules.push(textblockTypeInputRule(/^```([A-Za-z0-9_+-]*)\s$/, codeBlockType, (match) => {
			return { language: (match[1] || "").trim() };
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

const buildEditorPlugins = (schema: Schema) => {
	return [
		keymap(buildKeymap(schema)),
		keymap(baseKeymap),
		dropCursor(),
		gapCursor(),
		history(),
		keymap({
			"Mod-k": addLink,
			"Mod-Shift-k": removeLink
		}),
		buildInputRules(schema)
	];
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
		plugins: buildEditorPlugins(ZealotSchema)
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

const buildCodeBlockNodeView = (node: PMNode, view: EditorView, getPos: (() => number | undefined) | boolean): NodeView => {
	let currentNode = node;

	const dom = document.createElement("div");
	dom.className = "zealot-code-block";

	const toolbar = document.createElement("div");
	toolbar.className = "zealot-code-block-toolbar";

	const label = document.createElement("label");
	label.className = "zealot-code-block-language-label";
	label.textContent = "Code language";
	toolbar.appendChild(label);

	const languageSelect = document.createElement("select");
	languageSelect.className = "zealot-code-block-language-input";
	for (const optionDef of CODE_LANGUAGE_OPTIONS) {
		const option = document.createElement("option");
		option.value = optionDef.value;
		option.textContent = optionDef.label;
		languageSelect.appendChild(option);
	}
	toolbar.appendChild(languageSelect);

	const copyButton = document.createElement("button");
	copyButton.className = "zealot-code-block-copy";
	copyButton.type = "button";
	copyButton.textContent = "Copy";
	toolbar.appendChild(copyButton);

	let customLanguageOption: HTMLOptionElement | null = null;
	let copyTimer: number | null = null;

	const pre = document.createElement("pre");
	const code = document.createElement("code");
	pre.appendChild(code);

	const previewPre = document.createElement("pre");
	previewPre.className = "zealot-code-block-preview hljs";
	previewPre.setAttribute("aria-hidden", "true");
	const previewCode = document.createElement("code");
	previewPre.appendChild(previewCode);

	dom.appendChild(toolbar);
	dom.appendChild(previewPre);
	dom.appendChild(pre);

	const syncLanguageUi = (targetNode: PMNode) => {
		const language = (targetNode.attrs.language || "").trim();
		const knownOption = CODE_LANGUAGE_OPTIONS.some((entry) => entry.value === language);

		if (!knownOption && language.length > 0) {
			if (!customLanguageOption) {
				customLanguageOption = document.createElement("option");
				languageSelect.appendChild(customLanguageOption);
			}
			customLanguageOption.value = language;
			customLanguageOption.textContent = `${language} (custom)`;
		} else if (customLanguageOption) {
			customLanguageOption.remove();
			customLanguageOption = null;
		}

		if (languageSelect.value !== language) {
			languageSelect.value = language;
		}
		if (language.length > 0) {
			pre.setAttribute("data-language", language);
			code.className = `language-${language}`;
		} else {
			pre.removeAttribute("data-language");
			code.removeAttribute("class");
		}
	};

	const syncHighlightedPreview = (targetNode: PMNode) => {
		const text = targetNode.textContent || "";
		const language = (targetNode.attrs.language || "").trim().toLowerCase();

		if (language.length > 0 && hljs.getLanguage(language)) {
			const highlighted = hljs.highlight(text, {
				language,
				ignoreIllegals: true
			}).value;
			previewCode.innerHTML = highlighted;
			previewCode.className = `language-${language}`;
			return;
		}

		previewCode.textContent = text;
		previewCode.removeAttribute("class");
	};

	const applyLanguage = () => {
		if (typeof getPos !== "function") return;
		const pos = getPos();
		if (typeof pos !== "number") return;
		const targetNode = view.state.doc.nodeAt(pos);
		if (!targetNode || targetNode.type.name !== "code_block") return;

		const nextLanguage = languageSelect.value.trim();
		const currentLanguage = (targetNode.attrs.language || "").trim();
		if (nextLanguage === currentLanguage) return;

		const attrs = {
			...targetNode.attrs,
			language: nextLanguage
		};
		view.dispatch(view.state.tr.setNodeMarkup(pos, undefined, attrs, targetNode.marks));
	};

	languageSelect.addEventListener("mousedown", (event) => {
		event.stopPropagation();
	});

	languageSelect.addEventListener("change", () => {
		applyLanguage();
	});

	languageSelect.addEventListener("blur", () => {
		applyLanguage();
	});

	copyButton.addEventListener("mousedown", (event) => {
		event.stopPropagation();
	});

	copyButton.addEventListener("click", async (event) => {
		event.preventDefault();
		event.stopPropagation();

		const codeText = currentNode.textContent || "";
		try {
			if (navigator.clipboard?.writeText) {
				await navigator.clipboard.writeText(codeText);
			} else {
				const textarea = document.createElement("textarea");
				textarea.value = codeText;
				textarea.style.position = "fixed";
				textarea.style.opacity = "0";
				document.body.appendChild(textarea);
				textarea.focus();
				textarea.select();
				document.execCommand("copy");
				textarea.remove();
			}

			copyButton.textContent = "Copied";
			Popups.add("Copied code to clipboard")
			if (copyTimer !== null) window.clearTimeout(copyTimer);
			copyTimer = window.setTimeout(() => {
				copyButton.textContent = "Copy";
				copyTimer = null;
			}, 1200);
		} catch (error) {
			console.error("Failed to copy code block", error);
			copyButton.textContent = "Failed";
			if (copyTimer !== null) window.clearTimeout(copyTimer);
			copyTimer = window.setTimeout(() => {
				copyButton.textContent = "Copy";
				copyTimer = null;
			}, 1200);
		}
	});

	syncLanguageUi(currentNode);
	syncHighlightedPreview(currentNode);

	return {
		dom,
		contentDOM: code,
		update(nextNode) {
			if (nextNode.type !== currentNode.type) return false;
			currentNode = nextNode;
			syncLanguageUi(currentNode);
			syncHighlightedPreview(currentNode);
			return true;
		},
		stopEvent(event) {
			const target = event.target as HTMLElement | null;
			return Boolean(target?.closest(".zealot-code-block-toolbar"));
		}
	};
}

type WikiSuggestionContext = {
	from: number;
	to: number;
	query: string;
	anchorPos: number;
};

const getWikiSuggestionContext = (state: EditorState): WikiSuggestionContext | null => {
	if (!state.selection.empty) return null;

	const { $from } = state.selection;
	if (!$from.parent.isTextblock) return null;
	if ($from.parent.type.name === "code_block") return null;

	const textBefore = $from.parent.textBetween(0, $from.parentOffset, undefined, "\uFFFC");
	const match = /\[\[([^\]\n]*)$/.exec(textBefore);
	if (!match) return null;

	const query = (match[1] || "").trim();
	if (query.length === 0) return null;

	const startInParent = textBefore.length - match[0].length;
	const from = $from.start() + startInParent;
	const to = $from.pos;

	return { from, to, query, anchorPos: to };
}

export const createZealotEditorView = (
	host: HTMLElement,
	options: ZealotEditorOptions = {}
) => {
	const initial = options.content ?? "";
	const state = createZealotEditorState(initial);
	let saveTimer: number | null = null;
	let lastValue = initial;
	let wikiSuggestContext: WikiSuggestionContext | null = null;
	let wikiSuggestItems: Item[] = [];
	let wikiSuggestIndex = 0;
	let wikiSuggestReqId = 0;
	let wikiSuggestTimer: number | null = null;

	const wikiSuggest = document.createElement("div");
	wikiSuggest.className = "zealot-wiki-suggest";
	wikiSuggest.hidden = true;

	const wikiSuggestList = document.createElement("div");
	wikiSuggestList.className = "zealot-wiki-suggest-list";
	wikiSuggest.appendChild(wikiSuggestList);

	if (window.getComputedStyle(host).position === "static") {
		host.style.position = "relative";
	}
	host.appendChild(wikiSuggest);

	const closeWikiSuggest = () => {
		wikiSuggest.hidden = true;
		wikiSuggestContext = null;
		wikiSuggestItems = [];
		wikiSuggestIndex = 0;
		wikiSuggestList.innerHTML = "";
	}

	const applyWikiSuggestion = (index: number) => {
		if (!wikiSuggestContext) return false;
		if (index < 0 || index >= wikiSuggestItems.length) return false;

		const item = wikiSuggestItems[index];
		const title = (item.title || "").trim();
		if (title.length === 0) return false;

		const href = `/item/${title}`;
		const node = view.state.schema.nodes.itemlink.create({ title, href });
		const tr = view.state.tr.replaceWith(wikiSuggestContext.from, wikiSuggestContext.to, node);
		const insertPos = tr.mapping.map(wikiSuggestContext.from) + node.nodeSize;
		view.dispatch(tr.insertText(" ", insertPos));
		closeWikiSuggest();
		return true;
	}

	const renderWikiSuggest = () => {
		if (!wikiSuggestContext || wikiSuggestItems.length === 0) {
			closeWikiSuggest();
			return;
		}

		wikiSuggestList.innerHTML = "";
		wikiSuggestItems.forEach((item, index) => {
			const row = document.createElement("button");
			row.type = "button";
			row.className = "zealot-wiki-suggest-item";
			if (index === wikiSuggestIndex) {
				row.classList.add("selected");
			}

			const icon = (item.attributes?.["Icon"] || "").trim();
			const label = icon.length > 0 ? `${icon} ${item.title}` : item.title;
			row.textContent = label;

			row.addEventListener("mousedown", (event) => {
				event.preventDefault();
			});
			row.addEventListener("click", () => {
				applyWikiSuggestion(index);
				view.focus();
			});
			wikiSuggestList.appendChild(row);
		});

		const coords = view.coordsAtPos(wikiSuggestContext.anchorPos);
		const hostRect = host.getBoundingClientRect();
		const top = coords.bottom - hostRect.top + host.scrollTop + 6;
		const left = coords.left - hostRect.left + host.scrollLeft;
		wikiSuggest.style.top = `${top}px`;
		wikiSuggest.style.left = `${Math.max(0, left)}px`;
		wikiSuggest.hidden = false;
	}

	const scheduleWikiSuggestSearch = (term: string) => {
		if (wikiSuggestTimer !== null) {
			window.clearTimeout(wikiSuggestTimer);
			wikiSuggestTimer = null;
		}

		wikiSuggestTimer = window.setTimeout(async () => {
			wikiSuggestTimer = null;
			const reqId = ++wikiSuggestReqId;
			try {
				const results = await API.item.search(term);
				if (reqId !== wikiSuggestReqId) return;

				wikiSuggestItems = (results || []).slice(0, 8);
				wikiSuggestIndex = 0;
				renderWikiSuggest();
			} catch (error) {
				console.error("Failed wiki item search", error);
				closeWikiSuggest();
			}
		}, 120);
	}

	const refreshWikiSuggest = () => {
		const context = getWikiSuggestionContext(view.state);
		if (!context || !view.hasFocus()) {
			closeWikiSuggest();
			return;
		}

		const queryChanged = context.query !== wikiSuggestContext?.query;
		wikiSuggestContext = context;
		if (queryChanged || wikiSuggestItems.length === 0) {
			scheduleWikiSuggestSearch(context.query);
			return;
		}
		renderWikiSuggest();
	}

	const view = new EditorView(host, {
			state,
			nodeViews: {
				code_block: (node, view, getPos) => buildCodeBlockNodeView(node, view, getPos)
			},
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
				if (!wikiSuggest.hidden && wikiSuggestItems.length > 0) {
					if (event.key === "ArrowDown") {
						event.preventDefault();
						wikiSuggestIndex = Math.min(wikiSuggestIndex + 1, wikiSuggestItems.length - 1);
						renderWikiSuggest();
						return true;
					}
					if (event.key === "ArrowUp") {
						event.preventDefault();
						wikiSuggestIndex = Math.max(wikiSuggestIndex - 1, 0);
						renderWikiSuggest();
						return true;
					}
					if (event.key === "Enter" || event.key === "Tab") {
						event.preventDefault();
						return applyWikiSuggestion(wikiSuggestIndex);
					}
					if (event.key === "Escape") {
						event.preventDefault();
						closeWikiSuggest();
						return true;
					}
				}

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
			refreshWikiSuggest();

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

	view.dom.addEventListener("focusin", () => {
		refreshWikiSuggest();
	});

	view.dom.addEventListener("focusout", () => {
		window.setTimeout(() => {
			if (!view.hasFocus()) closeWikiSuggest();
		}, 0);
	});

	host.addEventListener("scroll", () => {
		if (wikiSuggest.hidden) return;
		renderWikiSuggest();
	});

	view.dom.setAttribute("name", "editor_prose");
	return view;
}

export default createZealotEditorView;
