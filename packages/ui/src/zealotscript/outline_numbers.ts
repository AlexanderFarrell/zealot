import type { Node as PMNode } from "prosemirror-model";
import { Plugin } from "prosemirror-state";
import { Decoration, DecorationSet } from "prosemirror-view";

export type OutlineLabel = { pos: number; label: string };

// "1." for a top-level item, "1.2" / "1.2.1" below it. Matches what the serializer writes.
export const outlineLabel = (path: number[]): string => path.join(".") + (path.length === 1 ? "." : "");

const labelList = (list: PMNode, listPos: number, path: number[], out: OutlineLabel[]): void => {
	let n = 0;
	list.forEach((item, offset) => {
		n++;
		const itemPos = listPos + 1 + offset;
		const itemPath = [...path, n];
		out.push({ pos: itemPos, label: outlineLabel(itemPath) });
		item.forEach((child, childOffset) => {
			if (child.type.name === "outline_list") labelList(child, itemPos + 1 + childOffset, itemPath, out);
		});
	});
};

// Every list_item inside an outline_list with its computed number. Numbers are never stored;
// inserting, deleting or moving an item renumbers because this runs on the current document.
export const computeOutlineLabels = (doc: PMNode): OutlineLabel[] => {
	const out: OutlineLabel[] = [];
	doc.descendants((node, pos) => {
		if (node.type.name !== "outline_list") return true;
		const $pos = doc.resolve(pos);
		const insideOutline = $pos.parent.type.name === "list_item" && $pos.depth >= 2 && $pos.node($pos.depth - 1).type.name === "outline_list";
		if (insideOutline) return false;
		labelList(node, pos, [], out);
		return false;
	});
	return out;
};

export const outlineNumbersPlugin = (): Plugin =>
	new Plugin({
		props: {
			decorations(state) {
				const labels = computeOutlineLabels(state.doc);
				if (labels.length === 0) return null;
				const decos = labels.map(({ pos, label }) => {
					const item = state.doc.nodeAt(pos);
					// --outline-w reserves room for the number, which grows with depth.
					return Decoration.node(pos, pos + (item?.nodeSize ?? 0), {
						"data-outline-number": label,
						style: `--outline-w: ${label.length + 1}ch`,
					});
				});
				return DecorationSet.create(state.doc, decos);
			},
		},
	});
