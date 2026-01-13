import type {Node as PMNode, Schema} from "prosemirror-model";
import { getCommandBlock } from "./commands/commands";

type InlineToken = {text: string};

const parseInline = (schema: Schema, text: string): PMNode => {
	// Minimal 
	return schema.nodes.paragraph.createAndFill({}, schema.text(text))!;
}

const parseParagraphs = (schema: Schema, lines: string[]): PMNode[] => {
	const blocks: PMNode[] = [];
	const buffer: string[] = [];

	const flush = () => {
		if (buffer.length === 0) return;
		const text = buffer.join("\n").trimEnd();
		if (text.trim().length > 0) {
			blocks.push(parseInline(schema, text));
		}
		buffer.length = 0;
	}

	for (const line of lines) {
		if (line.trim() === "") {
			flush();
		} else {
			buffer.push(line);
		}
	}

	flush();
	return blocks;
}

const parseHeading = (schema: Schema, line: string): PMNode | null => {
	const match = /^(#{1,6})\s+(.*)$/.exec(line);
	if (!match) return null;

	const level = match[1].length;
	const text = match[2];
	return schema.nodes.heading.createAndFill({ level }, schema.text(text));
}

const makeParagraph = (schema: Schema, text: string) => {
	if (text.trim().length === 0) return schema.nodes.paragraph.create();
	return schema.nodes.paragraph.create(null, schema.text(text));
}

const parseBulletList = (schema: Schema, lines: string[], startIndex: number) => {
	const items: PMNode[] = [];
	let index = startIndex;

	while (index < lines.length) {
		const line = lines[index];
		const match = /^-\s+(.+)$/.exec(line);
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

	const list = schema.nodes.ordered_list.create(null, items);
	if (!list) return null;

	return {node: list, linesConsumed: index - startIndex};
}

const blockTypes: Array<(schema: Schema, line: string) => PMNode | null> = [
	parseHeading
]

const multiblockTypes: Array<(schema: Schema, lines: string[], startIndex: number) => {node: PMNode, linesConsumed: number} | null> = [
	parseBulletList,
	parseOrderedList
]

export const parseZealotScript = (schema: Schema, input: string): PMNode => {
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
				const result = blockDef.parse(schema, lines, index + 1, args);
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

	return schema.nodes.doc.createAndFill({}, blocks)!;
}