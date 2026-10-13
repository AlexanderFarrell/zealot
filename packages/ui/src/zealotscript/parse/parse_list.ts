import type { Node as PMNode, Schema } from "prosemirror-model";
import { parseInline } from "./parse_inline";

type ListType = "bullet" | "ordered";

type ListInfo = {
	type: ListType;
	items: ListItemInfo[];
};

type ListItemInfo = {
	text: string;
	checked: boolean | null;
	children: ListInfo[];
};

type ParsedListLine = {
	indent: number;
	type: ListType;
	text: string;
	checked: boolean | null;
};

// Indent is measured in columns (tab = 4), not fixed levels, so Markdown that nests
// with 2 spaces (GitHub, Notion, ChatGPT) and tab-nested content both parse correctly.
const getIndentWidth = (prefix: string): number => {
	let width = 0;
	for (const c of prefix) {
		if (c === "\t") width += 4;
		else if (c === " ") width += 1;
	}
	return width;
};

const TASK_PREFIX_RE = /^\[([xX ])\](?:\s+|$)/;

const extractChecked = (text: string): { text: string; checked: boolean | null } => {
	const m = TASK_PREFIX_RE.exec(text);
	if (!m) return { text, checked: null };
	return { text: text.slice(m[0].length), checked: m[1] !== " " };
};

const parseListLine = (line: string): ParsedListLine | null => {
	// The marker may stand alone: an empty item is "- " (or "-" once trailing space is trimmed).
	const bulletMatch = /^([ \t]*)[-*+](?:[ \t]+(.*))?$/.exec(line);
	if (bulletMatch && !/^[ \t]*([-*+])\1{1,}[ \t]*$/.test(line)) {
		const { text, checked } = extractChecked((bulletMatch[2] ?? "").trim());
		return { indent: getIndentWidth(bulletMatch[1] ?? ""), type: "bullet", text, checked };
	}
	const orderedMatch = /^([ \t]*)\d+[.)](?:[ \t]+(.*))?$/.exec(line);
	if (orderedMatch) {
		const { text, checked } = extractChecked((orderedMatch[2] ?? "").trim());
		return { indent: getIndentWidth(orderedMatch[1] ?? ""), type: "ordered", text, checked };
	}
	return null;
};

const buildListNode = (schema: Schema, info: ListInfo): PMNode => {
	const listItem = schema.nodes["list_item"];
	if (!listItem) throw new Error("Schema missing list_item node");

	const items = info.items.map((item) => {
		const content: PMNode[] = [parseInline(schema, item.text)];
		for (const child of item.children) {
			content.push(buildListNode(schema, child));
		}
		return listItem.create({ checked: item.checked }, content);
	});

	const listTypeNode =
		info.type === "ordered" ? schema.nodes["ordered_list"] : schema.nodes["bullet_list"];
	if (!listTypeNode) throw new Error(`Schema missing ${info.type}_list node`);
	return listTypeNode.create(null, items);
};

export const parseList = (schema: Schema, lines: string[], startIndex: number) => {
	let index = startIndex;
	let linesConsumed = 0;
	let root: ListInfo | null = null;
	let stack: Array<{ indent: number; list: ListInfo }> = [];

	while (index < lines.length) {
		const rawLine = lines[index];
		if (rawLine === undefined) break;
		const parsed = parseListLine(rawLine);
		if (!parsed) {
			// A more-indented, non-blank line directly under an item continues that item.
			const continuation = /^([ \t]+)(\S.*)$/.exec(rawLine);
			const lastTop = stack[stack.length - 1];
			const lastItem = lastTop?.list.items[lastTop.list.items.length - 1];
			if (!continuation || !lastItem || !lastTop) break;
			if (getIndentWidth(continuation[1] ?? "") <= lastTop.indent) break;
			if (/^(```|:::|#{1,6}\s|>)/.test(continuation[2] ?? "")) break;
			lastItem.text = lastItem.text.length > 0 ? `${lastItem.text} ${continuation[2]}` : (continuation[2] ?? "");
			index++;
			linesConsumed++;
			continue;
		}

		if (!root) {
			root = { type: parsed.type, items: [] };
			stack = [{ indent: parsed.indent, list: root }];
		}

		const baseIndent = stack[0]?.indent ?? 0;
		if (parsed.indent < baseIndent) break;
		if (parsed.indent === baseIndent && parsed.type !== root.type) break;

		while (stack.length > 1 && parsed.indent < (stack[stack.length - 1]?.indent ?? 0)) {
			stack.pop();
		}

		const topOfStack = stack[stack.length - 1];
		if (!topOfStack) break;

		if (parsed.indent > topOfStack.indent) {
			const parentList = topOfStack.list;
			const lastItem = parentList.items[parentList.items.length - 1];
			if (!lastItem) break;
			const childList: ListInfo = { type: parsed.type, items: [] };
			lastItem.children.push(childList);
			stack.push({ indent: parsed.indent, list: childList });
		} else if (parsed.indent === topOfStack.indent && parsed.type !== topOfStack.list.type) {
			const parentEntry = stack.length > 1 ? stack[stack.length - 2] : null;
			if (!parentEntry) break;
			const lastItem = parentEntry.list.items[parentEntry.list.items.length - 1];
			if (!lastItem) break;
			const newList: ListInfo = { type: parsed.type, items: [] };
			lastItem.children.push(newList);
			stack[stack.length - 1] = { indent: parsed.indent, list: newList };
		}

		const currentTop = stack[stack.length - 1];
		if (!currentTop) break;
		currentTop.list.items.push({ text: parsed.text, checked: parsed.checked, children: [] });
		index++;
		linesConsumed++;
	}

	if (!root || root.items.length === 0) return null;
	return { node: buildListNode(schema, root), linesConsumed };
};
