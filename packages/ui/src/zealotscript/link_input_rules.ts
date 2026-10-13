import { InputRule } from "prosemirror-inputrules";
import type { Schema } from "prosemirror-model";

// Typing [[Title]] makes a wiki link straight away instead of waiting for a reload.
export const wikilinkHref = (target: string, anchor?: string): string => {
	const isType = target.startsWith("type:");
	const base = isType ? `zealot://type/${target.slice("type:".length)}` : `zealot://item/${target}`;
	return anchor ? `${base}#${anchor}` : base;
};

export const WIKILINK_TYPED_RE = /\[\[([^[\]|#]+)(?:#([^\]|]+))?(?:\|([^\]]+))?\]\]$/;

export const wikilinkInputRule = (schema: Schema): InputRule | null => {
	const linkMark = schema.marks["link"];
	if (!linkMark) return null;
	return new InputRule(WIKILINK_TYPED_RE, (state, match, start, end) => {
		const target = (match[1] ?? "").trim();
		if (target.length === 0) return null;
		const label = (match[3] ?? (target.startsWith("type:") ? target.slice("type:".length) : target)).trim();
		const mark = linkMark.create({ href: wikilinkHref(target, match[2]) });
		return state.tr.replaceWith(start, end, schema.text(label, [mark]));
	});
};

// "[ ] " at the start of a list item turns that item into a task. The "- [ ] " form
// can never match because "- " has already become a bullet by then.
export const taskItemInputRule = (schema: Schema): InputRule | null => {
	const listItem = schema.nodes["list_item"];
	if (!listItem) return null;
	return new InputRule(/^\[([xX ])\]\s$/, (state, match, start, end) => {
		const $start = state.doc.resolve(start);
		if ($start.depth < 2 || $start.node($start.depth - 1).type !== listItem) return null;
		if ($start.index($start.depth - 1) !== 0) return null;
		const itemPos = $start.before($start.depth - 1);
		const item = $start.node($start.depth - 1);
		const tr = state.tr.delete(start, end);
		return tr.setNodeMarkup(itemPos, undefined, { ...item.attrs, checked: (match[1] ?? " ") !== " " });
	});
};
