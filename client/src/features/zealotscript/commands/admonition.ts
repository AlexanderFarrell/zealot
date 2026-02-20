import type { ZCommandBlockInfo } from "./types";
import type {Node as PMNode, Schema} from "prosemirror-model";
import { parseCodeBlock } from "../parse/parse_code_block";
import { parseInlineNodes } from "../parse/parse_inline";
import { parseList } from "../parse/parse_list";
import { parseParagraphs } from "../parse/parse_paragraph";
import { parseMarkdownTable } from "../parse/parse_table";

const ADMONITION_KINDS = new Set([
	"note",
	"warning",
	"danger",
	"tip",
	"info",
	"success",
	"important",
	"caution",
	"example",
	"faq",
	"todo"
]);

const parseAdmonitionHeading = (schema: Schema, line: string): PMNode | null => {
	const match = /^(#{1,6})\s+(.*)$/.exec(line);
	if (!match) return null;
	return schema.nodes.heading.create({ level: match[1].length }, parseInlineNodes(schema, match[2]));
}

const parseAdmonitionContent = (schema: Schema, lines: string[]): PMNode[] => {
	const blocks: PMNode[] = [];
	let index = 0;

	while (index < lines.length) {
		const line = lines[index];

		const heading = parseAdmonitionHeading(schema, line);
		if (heading) {
			blocks.push(heading);
			index++;
			continue;
		}

		const list = parseList(schema, lines, index);
		if (list) {
			blocks.push(list.node);
			index += list.linesConsumed;
			continue;
		}

		const code = parseCodeBlock(schema, lines, index);
		if (code) {
			blocks.push(code.node);
			index += code.linesConsumed;
			continue;
		}

		const markdownTable = parseMarkdownTable(schema, lines, index);
		if (markdownTable) {
			blocks.push(markdownTable.node);
			index += markdownTable.linesConsumed;
			continue;
		}

		blocks.push(...parseParagraphs(schema, [line]));
		index++;
	}

	return blocks;
}

const parseAdmonitionBlock: ZCommandBlockInfo["parse"] = (schema, lines, startIndex, args) => {
	const declaredKind = (args[1] || args[0] || "note").toLowerCase();
	const kind = ADMONITION_KINDS.has(declaredKind) ? declaredKind : "note";
	const contentLines: string[] = [];
	let index = startIndex;

	while (index < lines.length) {
		const line = lines[index];
		if (line.trim() === ":::") break;
		contentLines.push(line);
		index++;
	}

	const blocks = parseAdmonitionContent(schema, contentLines);
	if (blocks.length === 0) {
		blocks.push(schema.nodes.paragraph.create());
	}

	const admonition = schema.nodes.admonition.createAndFill({kind}, blocks);
	if (!admonition) return null;

	const hasClosingFence = index < lines.length && lines[index].trim() === ":::";
	return {node: admonition, linesConsumed: index - startIndex + (hasClosingFence ? 1 : 0)};
}

const serializeAdmonition: ZCommandBlockInfo["serialize"] = (node) => {
	if (node.type.name !== "admonition") return null;

	const kind = node.attrs.kind || "note";
	const lines: string[] = [];
	lines.push(`:::${kind}`);
	lines.push(node.textContent || "");
	lines.push(":::");
	return lines.join("\n");
}

export const noteAdmonitionBlock: ZCommandBlockInfo = {
	name: "note",
	parse: parseAdmonitionBlock,
	serialize: serializeAdmonition,
}

export const warningAdmonitionBlock: ZCommandBlockInfo = {
	name: "warning",
	parse: parseAdmonitionBlock,
	serialize: serializeAdmonition,
}

export const dangerAdmonitionBlock: ZCommandBlockInfo = {
	name: "danger",
	parse: parseAdmonitionBlock,
	serialize: serializeAdmonition
}

export const tipAdmonitionBlock: ZCommandBlockInfo = {
	name: "tip",
	parse: parseAdmonitionBlock,
	serialize: serializeAdmonition
}

export const infoAdmonitionBlock: ZCommandBlockInfo = {
	name: "info",
	parse: parseAdmonitionBlock,
	serialize: serializeAdmonition
}

export const successAdmonitionBlock: ZCommandBlockInfo = {
	name: "success",
	parse: parseAdmonitionBlock,
	serialize: serializeAdmonition
}

export const importantAdmonitionBlock: ZCommandBlockInfo = {
	name: "important",
	parse: parseAdmonitionBlock,
	serialize: serializeAdmonition
}

export const cautionAdmonitionBlock: ZCommandBlockInfo = {
	name: "caution",
	parse: parseAdmonitionBlock,
	serialize: serializeAdmonition
}

export const exampleAdmonitionBlock: ZCommandBlockInfo = {
	name: "example",
	parse: parseAdmonitionBlock,
	serialize: serializeAdmonition
}

export const faqAdmonitionBlock: ZCommandBlockInfo = {
	name: "faq",
	parse: parseAdmonitionBlock,
	serialize: serializeAdmonition
}

export const todoAdmonitionBlock: ZCommandBlockInfo = {
	name: "todo",
	parse: parseAdmonitionBlock,
	serialize: serializeAdmonition
}
