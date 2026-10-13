import { Plugin, PluginKey, type EditorState } from "prosemirror-state";
import type { Node as PMNode, Schema } from "prosemirror-model";
import { hrefForUrl, isUrlStart, matchBareUrl } from "./linkify";

const MD_LINK_RE = /\[([^\]\n]+)\]\(([^)\s]+)\)/g;

type Edit = { from: number; to: number; href: string; label: string | null };

// Collects links to create inside one textblock. A candidate only counts as finished
// when whitespace follows it, or it ends the block and the cursor has moved on, so a
// URL is never linked half-typed.
export const findLinkEdits = (block: PMNode, blockPos: number, cursor: number, schema: Schema): Edit[] => {
	const linkType = schema.marks["link"];
	const codeType = schema.marks["code"];
	if (!linkType) return [];

	let text = "";
	const blocked: boolean[] = [];
	block.forEach((child) => {
		const skip = child.marks.some((m) => m.type === linkType || m.type === codeType);
		const chunk = child.isText ? (child.text ?? "") : "\uFFFC";
		text += chunk;
		for (let i = 0; i < chunk.length; i++) blocked.push(skip || !child.isText);
	});

	const start = blockPos + 1;
	// Finished = whitespace or block end follows, and the cursor is not inside or
	// touching the candidate (typing into "[a](|)" must not link on the first character).
	const settled = (from: number, end: number): boolean => {
		if (cursor >= start + from && cursor <= start + end) return false;
		const after = text[end];
		return after === undefined || /\s/.test(after);
	};
	const free = (from: number, to: number): boolean => {
		for (let i = from; i < to; i++) if (blocked[i]) return false;
		return true;
	};

	const edits: Edit[] = [];
	const covered: Array<[number, number]> = [];

	MD_LINK_RE.lastIndex = 0;
	for (let m = MD_LINK_RE.exec(text); m; m = MD_LINK_RE.exec(text)) {
		const from = m.index;
		const to = from + m[0].length;
		if (text[from - 1] === "!") continue;
		covered.push([from, to]);
		if (!free(from, to) || !settled(from, to)) continue;
		edits.push({ from: start + from, to: start + to, href: m[2] ?? "", label: m[1] ?? "" });
	}

	for (let i = 0; i < text.length; i++) {
		if (covered.some(([a, b]) => i >= a && i < b)) continue;
		if (!isUrlStart(text, i)) continue;
		const url = matchBareUrl(text.slice(i));
		if (!url) continue;
		const end = i + url.length;
		if (free(i, end) && settled(i, end)) {
			edits.push({ from: start + i, to: start + end, href: hrefForUrl(url), label: null });
		}
		i = end - 1;
	}
	return edits;
};

export const collectLinkEdits = (state: EditorState, ranges: Array<[number, number]>): Edit[] => {
	const seen = new Set<number>();
	const edits: Edit[] = [];
	const cursor = state.selection.from;
	for (const [rawFrom, rawTo] of ranges) {
		const from = Math.max(0, rawFrom - 1);
		const to = Math.min(state.doc.content.size, rawTo + 1);
		state.doc.nodesBetween(from, to, (node, pos) => {
			if (!node.isTextblock) return true;
			if (seen.has(pos)) return false;
			seen.add(pos);
			if (node.type.spec.code) return false;
			edits.push(...findLinkEdits(node, pos, cursor, state.schema));
			return false;
		});
	}
	return edits.sort((a, b) => b.from - a.from);
};

// Turns finished bare URLs and [label](url) text into link marks as the user types,
// including when Enter ends a list item or paragraph with no trailing space.
export const autolinkPlugin = (): Plugin =>
	new Plugin({
		key: new PluginKey("zealotAutolink"),
		appendTransaction: (transactions, _oldState, newState) => {
			if (!transactions.some((tr) => tr.docChanged)) return null;
			const linkType = newState.schema.marks["link"];
			if (!linkType) return null;

			const ranges: Array<[number, number]> = [];
			for (const tr of transactions) {
				if (!tr.docChanged) continue;
				tr.mapping.maps.forEach((map, i) => {
					map.forEach((_oldStart, _oldEnd, newStart, newEnd) => {
						const rest = tr.mapping.slice(i + 1);
						ranges.push([rest.map(newStart, -1), rest.map(newEnd, 1)]);
					});
				});
			}
			// Later transactions in the batch may have shortened the document.
			const size = newState.doc.content.size;
			const clamped = ranges.map(([a, b]): [number, number] => [Math.min(a, size), Math.min(b, size)]);
			const edits = collectLinkEdits(newState, clamped);
			if (edits.length === 0) return null;

			const tr = newState.tr;
			for (const edit of edits) {
				const mark = linkType.create({ href: edit.href });
				if (edit.label === null) {
					tr.addMark(edit.from, edit.to, mark);
				} else {
					tr.replaceWith(edit.from, edit.to, newState.schema.text(edit.label, [mark]));
				}
			}
			return tr;
		},
	});
