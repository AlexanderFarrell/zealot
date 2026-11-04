import type { Node as PMNode, Schema } from "prosemirror-model";
import { parseInline } from "./parse_inline";

type OutlineItemInfo = {
	text: string;
	checked: boolean | null;
	children: OutlineInfo;
};

type OutlineInfo = OutlineItemInfo[];

// Same column-based indent as parse_list.ts: tab = 4, so 2-space and tab nesting both work.
const getIndentWidth = (prefix: string): number => {
	let width = 0;
	for (const c of prefix) {
		if (c === "\t") width += 4;
		else if (c === " ") width += 1;
	}
	return width;
};

// A leading number ("1.", "1.2", "1.2.3.") is accepted and dropped: numbers are derived
// from position, so whatever was typed never decides what is displayed.
const OUTLINE_LINE_RE = /^([ \t]*)(?:\d+(?:\.\d+)*\.?(?=\s|$)[ \t]*)?(?:\[([xX ])\](?:[ \t]+|$))?(.*)$/;

const buildOutlineNode = (schema: Schema, items: OutlineInfo): PMNode => {
	const listItem = schema.nodes["list_item"];
	const outlineList = schema.nodes["outline_list"];
	if (!listItem || !outlineList) throw new Error("Schema missing outline_list or list_item node");

	const built = items.map((item) => {
		const content: PMNode[] = [parseInline(schema, item.text)];
		if (item.children.length > 0) content.push(buildOutlineNode(schema, item.children));
		return listItem.create({ checked: item.checked }, content);
	});
	return outlineList.create(null, built);
};

export const parseOutline = (schema: Schema, lines: string[], startIndex: number): { node: PMNode; linesConsumed: number } | null => {
	const openLine = lines[startIndex];
	if (openLine === undefined || !/^:::outline\s*$/.test(openLine)) return null;
	if (!schema.nodes["outline_list"]) return null;

	const root: OutlineInfo = [];
	const stack: Array<{ indent: number; items: OutlineInfo }> = [];
	let index = startIndex + 1;

	while (index < lines.length) {
		const line = lines[index];
		if (line === undefined) break;
		if (line.trim() === ":::") { index++; break; }
		index++;
		if (line.trim() === "") continue;

		const m = OUTLINE_LINE_RE.exec(line);
		if (!m) continue;
		const indent = getIndentWidth(m[1] ?? "");
		const item: OutlineItemInfo = { text: (m[3] ?? "").trim(), checked: m[2] === undefined ? null : m[2] !== " ", children: [] };

		if (stack.length === 0) {
			stack.push({ indent, items: root });
		} else {
			// Anything shallower than the first item is clamped to the top level.
			while (stack.length > 1 && indent < (stack[stack.length - 1]?.indent ?? 0)) stack.pop();
			const top = stack[stack.length - 1];
			if (top && indent > top.indent) {
				const parent = top.items[top.items.length - 1];
				if (parent) stack.push({ indent, items: parent.children });
			}
		}
		stack[stack.length - 1]?.items.push(item);
	}

	if (root.length === 0) root.push({ text: "", checked: null, children: [] });
	return { node: buildOutlineNode(schema, root), linesConsumed: index - startIndex };
};
