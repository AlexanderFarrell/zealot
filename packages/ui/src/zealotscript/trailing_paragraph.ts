import { Plugin, PluginKey } from "prosemirror-state";
import type { Node as PMNode } from "prosemirror-model";

// Blocks such as tables, code blocks, quotes and lists can end a document, leaving
// nowhere to click or press Enter to continue writing. The editor keeps an empty
// paragraph after them while editing and strips it again when saving.

export const needsTrailingParagraph = (doc: PMNode): boolean => {
	const last = doc.lastChild;
	return last !== null && last.type.name !== "paragraph" && last.type.name !== "heading";
};

export const ensureTrailingParagraph = (doc: PMNode): PMNode => {
	if (!needsTrailingParagraph(doc)) return doc;
	const paragraph = doc.type.schema.nodes["paragraph"];
	if (!paragraph) return doc;
	return doc.copy(doc.content.addToEnd(paragraph.create()));
};

// Drops the editing-only paragraph so saved content round-trips: an empty last
// paragraph that follows a non-paragraph block.
export const stripTrailingParagraph = (doc: PMNode): PMNode => {
	const count = doc.childCount;
	if (count < 2) return doc;
	const last = doc.child(count - 1);
	const prev = doc.child(count - 2);
	if (last.type.name !== "paragraph" || last.content.size > 0) return doc;
	if (prev.type.name === "paragraph" || prev.type.name === "heading") return doc;
	return doc.copy(doc.content.cut(0, doc.content.size - last.nodeSize));
};

export const trailingParagraphPlugin = (): Plugin =>
	new Plugin({
		key: new PluginKey("zealotTrailingParagraph"),
		appendTransaction: (_transactions, _oldState, newState) => {
			if (!needsTrailingParagraph(newState.doc)) return null;
			const paragraph = newState.schema.nodes["paragraph"];
			if (!paragraph) return null;
			return newState.tr.insert(newState.doc.content.size, paragraph.create()).setMeta("addToHistory", false);
		},
	});
