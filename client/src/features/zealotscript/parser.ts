import type {Node as PMNode, Schema} from "prosemirror-model";
import { getCommandBlock } from "./commands/commands";
import { makeParagraph, parseParagraphs } from "./parse/parse_paragraph";
import { parseList } from "./parse/parse_list";
import { parseCodeBlock } from "./parse/parse_code_block";
import { parseInlineNodes } from "./parse/parse_inline";
import { parseMarkdownTable } from "./parse/parse_table";

const isZealotDebugEnabled = () => {
	if (typeof window === "undefined") return false;
	return window.localStorage.getItem("zealotscript_debug") === "1";
}

const debugZealot = (...args: unknown[]) => {
	if (!isZealotDebugEnabled()) return;
	console.debug("[ZealotScript]", ...args);
}





const parseHeading = (schema: Schema, line: string): PMNode | null => {
	const match = /^(#{1,6})\s+(.*)$/.exec(line);
	if (!match) return null;

	const level = match[1].length;
	const text = match[2];
	return schema.nodes.heading.create({ level }, parseInlineNodes(schema, text));
}

const parseBlockquote = (schema: Schema, lines: string[], startIndex: number) => {
	const quoteLines: string[] = [];
	let index = startIndex;

	while (index < lines.length) {
		const line = lines[index];
		const match = /^[ \t]*>\s?(.*)$/.exec(line);
		if (!match) break;
		quoteLines.push(match[1]);
		index++;
	}

	if (quoteLines.length === 0) return null;

	const blocks = parseParagraphs(schema, quoteLines);
	const content = blocks.length > 0 ? blocks : [schema.nodes.paragraph.create()];
	const node = schema.nodes.blockquote.create(null, content);
	return { node, linesConsumed: index - startIndex };
}



const parseBulletList = (schema: Schema, lines: string[], startIndex: number) => {
	const items: PMNode[] = [];
	let index = startIndex;

	while (index < lines.length) {
		const line = lines[index];
		const match = /^[-*]\s+(.+)$/.exec(line);
		if (!match) break;

		const paragraph = makeParagraph(schema, match[1]);
		const listItem = schema.nodes.list_item.create(null, paragraph);
		items.push(listItem);
		index++;
		// const itemText = match[1];
		// const paragraph = 
		// 	itemText.trim().length > 0
		// 	? schema.nodes.paragraph.createAndFill({}, schema.text(itemText))!
		// 	: schema.nodes.paragraph.createAndFill()!;
		// const listItem = schema.nodes.list_item.createAndFill({}, paragraph)!;
		// items.push(listItem);
		// index++;
	}
	if (items.length === 0) return null;

	const list = schema.nodes.bullet_list.create(null, items);
	if (!list) return null;

	return {node: list, linesConsumed: index - startIndex};
}

const parseOrderedList = (schema: Schema, lines: string[], startIndex: number) => {
	const items: PMNode[] = [];
	let index = startIndex;

	while (index < lines.length) {
		const line = lines[index];
		const match = /^\d+\.\s+(.+)$/.exec(line);
		if (!match) break;

		// const itemText = match[1];
		const paragraph = makeParagraph(schema, match[1]);
		const listItem = schema.nodes.list_item.create(null, paragraph);
		items.push(listItem);
		index++;
		// const paragraph = schema.nodes.paragraph.createAndFill({}, schema.text(itemText))!;
		// const listItem = schema.nodes.list_item.createAndFill({}, paragraph)!;
		// items.push(listItem);
		// index++;
	}

	if (items.length === 0) return null;

	const list = schema.nodes.ordered_list.create(null, items);
	if (!list) return null;

	return {node: list, linesConsumed: index - startIndex};
}

const blockTypes: Array<(schema: Schema, line: string) => PMNode | null> = [
	parseHeading
]

const multiblockTypes: Array<(schema: Schema, lines: string[], startIndex: number) => {node: PMNode, linesConsumed: number} | null> = [
	parseBlockquote,
	parseMarkdownTable,
	// parseBulletList,
	parseList,
	parseCodeBlock
	// parseOrderedList
]

export const parseZealotScript = (schema: Schema, input: string): PMNode => {
	debugZealot("parse input", input);
	const lines = input.replace(/\r\n/g, "\n").split('\n');
	const blocks: PMNode[] = [];

	let index = 0;
	while (index < lines.length) {
		const line = lines[index];

		if (line.trim().startsWith(":::")) {
			const raw = line.trim().slice(3).trim()
			const [name, ...args] = raw.split(",").map((part) => part.trim()).filter(Boolean);
			const blockDef = name ? getCommandBlock(name) : null;

			if (blockDef) {
				const result = blockDef.parse(schema, lines, index + 1, [name, ...args]);
				if (result) {
					blocks.push(result.node);
					index = index + 1 + result.linesConsumed;
					continue;
				}
			}
		}

		let done: boolean = false;
		for (let i = 0; i < blockTypes.length; i++) {
			const t = blockTypes[i];
			const node = t(schema, line)
			if (node) {
				blocks.push(node)
				index++;
				done = true;
				break;
			}
		}

		if (done) {
			continue;
		}

		for (let i = 0; i < multiblockTypes.length; i++) {
			const t = multiblockTypes[i];
			const node = t(schema, lines, index);
			if (node) {
				blocks.push(node.node);
				index += node.linesConsumed;
				done = true;
				break;
			}
		}

		if (done) {
			continue;
		}

		const paragraphs = parseParagraphs(schema, [line]);
		blocks.push(...paragraphs);
		index++;
	}

	const doc = schema.nodes.doc.createAndFill({}, blocks)!;
	debugZealot("parse output", JSON.stringify(doc.toJSON()));
	return doc;
}
