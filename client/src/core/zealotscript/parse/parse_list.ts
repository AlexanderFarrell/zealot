import type {Node as PMNode, Schema} from "prosemirror-model";
import { makeParagraph } from "./parse_paragraph";
import { parseInline } from "./parse_inline";

type ListType = "bullet" | "ordered";

type ListInfo = {
	type: ListType,
	items: ListItemInfo[];
}

type ListItemInfo = {
	text: string,
	children: ListInfo[];
}

type ParsedListLine = {
	indent: number;
	type: ListType;
	text: string;
}

const getIndentLevel = (prefix: string) => {
	let tabs = 0;
	let spaces = 0;
	for (const c of prefix) {
		if (c === "\t") tabs++;
		else if (c === " ") spaces++;
	}
	return tabs + Math.floor(spaces / 4);
}

const parseListLine = (line: string): ParsedListLine | null => {
	const bulletMatch = /^([ \t]*)([-*])\s+(.+)$/.exec(line);
	if (bulletMatch) {
		return {
			indent: getIndentLevel(bulletMatch[1]),
			type: "bullet",
			text: bulletMatch[3]
		};
	}

	const orderedMatch = /^([ \t]*)(\d+)\.\s+(.+)$/.exec(line);
	if (orderedMatch) {
		return {
			indent: getIndentLevel(orderedMatch[1]),
			type: 'ordered',
			text: orderedMatch[3]
		};
	}

	return null;
}

const buildListNode = (schema: Schema, info: ListInfo): PMNode => {
	const items = info.items.map((item) => {
		const content: PMNode[] = [parseInline(schema, item.text)];
		for (const child of item.children) {
			content.push(buildListNode(schema, child));
		}
		return schema.nodes.list_item.create(null, content);
	})

	const listTypeNode = 
		info.type === "ordered" ? schema.nodes.ordered_list : schema.nodes.bullet_list;

	return listTypeNode.create(null, items);
}

export const parseList = (schema: Schema, lines: string[], startIndex: number) => {
	let index = startIndex;
	let linesConsumed = 0;

	let root: ListInfo | null = null;
	let stack: Array<{indent: number, list: ListInfo}> = [];

	while (index < lines.length) {
		const parsed = parseListLine(lines[index]);
		if (!parsed) break;

		if (!root) {
			root = {type: parsed.type, items: []};
			stack = [{indent: parsed.indent, list: root}];
		}

		const baseIndent = stack[0].indent;
		if (parsed.indent < baseIndent) break;
		if (parsed.indent === baseIndent && parsed.type !== root.type) break;
		
		while (stack.length > 1 && parsed.indent < stack[stack.length - 1].indent) {
			stack.pop();
		}

		if (parsed.indent > stack[stack.length - 1].indent) {
			const parentList = stack[stack.length - 1].list;
			const lastItem = parentList.items[parentList.items.length - 1];
			if (!lastItem) break;

			const childList: ListInfo = {type: parsed.type, items: []};
			lastItem.children.push(childList);
			stack.push({indent: parsed.indent, list: childList});
		} else if (
			parsed.indent === stack[stack.length - 1].indent &&
			parsed.type !== stack[stack.length - 1].list.type
		) {
			const parent = stack.length > 1 ? stack[stack.length - 2].list : null;
			if (!parent) break;

			const lastItem = parent.items[parent.items.length - 1];
			if (!lastItem) break;

			const newList: ListInfo = {type: parsed.type, items: []};
			lastItem.children.push(newList);
			stack[stack.length - 1] = {indent: parsed.indent, list: newList};
		}

		const item: ListItemInfo = {text: parsed.text, children: []}
		stack[stack.length - 1].list.items.push(item);

		index++;
		linesConsumed++;
	}

	if (!root || root.items.length === 0) return null;

	return {node: buildListNode(schema, root), linesConsumed}
}