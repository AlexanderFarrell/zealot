// Live syntax highlighting for code blocks in the ZealotScript editor.
//
// IMPORTANT: unlike the read-only <zealotscript-view>, we must NOT rewrite the
// <pre><code> innerHTML here — that would desync ProseMirror's contenteditable
// model and corrupt the document/selection. Instead we tokenize with
// highlight.js and paint the tokens as ProseMirror inline Decorations, which
// style character ranges without touching the underlying document.
//
// highlight.js loads asynchronously, so the plugin recomputes once the library
// is ready (via a self-dispatched meta transaction).

import { Plugin, PluginKey } from "prosemirror-state";
import type { EditorState, Transaction } from "prosemirror-state";
import { Decoration, DecorationSet } from "prosemirror-view";
import type { EditorView } from "prosemirror-view";
import type { Node as PMNode } from "prosemirror-model";
import { highlightToTokens, highlighterOrNull, whenHighlighterReady } from "./highlight";

const codeHighlightKey = new PluginKey<DecorationSet>("zealot-code-highlight");

const buildDecorations = (doc: PMNode): DecorationSet => {
	if (highlighterOrNull() === null) return DecorationSet.empty;

	const decorations: Decoration[] = [];
	doc.descendants((node, pos) => {
		if (node.type.name !== "code_block") return;
		const language = (node.attrs.language as string | undefined) ?? "";
		if (language === "mermaid") return; // rendered as a diagram, not code
		const text = node.textContent;
		if (text.length === 0) return;

		const tokens = highlightToTokens(text, language);
		if (!tokens) return;

		// +1: the text inside a code_block starts one position after the node.
		const base = pos + 1;
		for (const token of tokens) {
			if (token.className.length === 0) continue;
			decorations.push(
				Decoration.inline(base + token.from, base + token.to, { class: token.className }),
			);
		}
	});

	return DecorationSet.create(doc, decorations);
};

export const codeHighlightPlugin = (): Plugin<DecorationSet> =>
	new Plugin<DecorationSet>({
		key: codeHighlightKey,
		state: {
			init: (_config, state) => buildDecorations(state.doc),
			apply: (tr: Transaction, value: DecorationSet, _old: EditorState, newState: EditorState) => {
				// Recompute on any doc change, or when the async lib-ready signal fires.
				if (tr.docChanged || tr.getMeta(codeHighlightKey) === "recompute") {
					return buildDecorations(newState.doc);
				}
				return value.map(tr.mapping, tr.doc);
			},
		},
		props: {
			decorations: (state) => codeHighlightKey.getState(state),
		},
		view: (view: EditorView) => {
			// Kick off the async load and re-decorate once tokens are available.
			void whenHighlighterReady().then(() => {
				if (view.isDestroyed) return;
				view.dispatch(view.state.tr.setMeta(codeHighlightKey, "recompute"));
			});
			return {};
		},
	});
