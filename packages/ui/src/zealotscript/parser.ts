import type { Node as PMNode, Schema } from "prosemirror-model";
import { parseParagraphs } from "./parse/parse_paragraph";
import { parseList } from "./parse/parse_list";
import { parseCodeBlock } from "./parse/parse_code_block";
import { parseInlineNodes } from "./parse/parse_inline";
import { parseMarkdownTable, parseFencedTable } from "./parse/parse_table";
import { parseYoutubeEmbed } from "./parse/parse_youtube";
import { parseMathBlock } from "./parse/parse_math_block";
import { parseDetails, parseSpoiler, parseDefinitionList, parseColumns, parseTabs } from "./parse/parse_structural_blocks";

const ADMONITION_KINDS = new Set([
	"note", "warning", "danger", "tip", "info",
	"success", "important", "caution", "example", "faq", "todo",
]);

const parseHeading = (schema: Schema, line: string): PMNode | null => {
	const match = /^(#{1,6})\s+(.*)$/.exec(line);
	if (!match) return null;
	const level = match[1]?.length ?? 1;
	const text = match[2] ?? "";
	const heading = schema.nodes["heading"];
	if (!heading) return null;
	return heading.create({ level }, parseInlineNodes(schema, text));
};

const parseAdmonitionContent = (schema: Schema, lines: string[]): PMNode[] => {
	const blocks: PMNode[] = [];
	let index = 0;
	while (index < lines.length) {
		const line = lines[index];
		if (line === undefined) { index++; continue; }

		const headingNode = parseHeading(schema, line);
		if (headingNode) { blocks.push(headingNode); index++; continue; }

		const list = parseList(schema, lines, index);
		if (list) { blocks.push(list.node); index += list.linesConsumed; continue; }

		const code = parseCodeBlock(schema, lines, index);
		if (code) { blocks.push(code.node); index += code.linesConsumed; continue; }

		const table = parseMarkdownTable(schema, lines, index);
		if (table) { blocks.push(table.node); index += table.linesConsumed; continue; }

		blocks.push(...parseParagraphs(schema, [line]));
		index++;
	}
	return blocks;
};

const parseAdmonitionBlock = (schema: Schema, lines: string[], startIndex: number) => {
	const openLine = lines[startIndex];
	if (!openLine) return null;
	const openMatch = /^:::([A-Za-z]+)\s*$/.exec(openLine);
	if (!openMatch) return null;
	const declaredKind = (openMatch[1] ?? "note").toLowerCase();
	const kind = ADMONITION_KINDS.has(declaredKind) ? declaredKind : "note";

	const admonitionNode = schema.nodes["admonition"];
	const paragraphNode = schema.nodes["paragraph"];
	if (!admonitionNode || !paragraphNode) return null;

	const contentLines: string[] = [];
	let index = startIndex + 1;
	while (index < lines.length) {
		const line = lines[index];
		if (line === undefined) break;
		if (line.trim() === ":::") { index++; break; }
		contentLines.push(line);
		index++;
	}

	const blocks = parseAdmonitionContent(schema, contentLines);
	const content = blocks.length > 0 ? blocks : [paragraphNode.create()];
	const node = admonitionNode.createAndFill({ kind }, content);
	if (!node) return null;
	return { node, linesConsumed: index - startIndex };
};

const parseBlockquote = (schema: Schema, lines: string[], startIndex: number) => {
	const quoteLines: string[] = [];
	let index = startIndex;

	while (index < lines.length) {
		const line = lines[index];
		if (line === undefined) break;
		const match = /^[ \t]*>\s?(.*)$/.exec(line);
		if (!match) break;
		quoteLines.push(match[1] ?? "");
		index++;
	}

	if (quoteLines.length === 0) return null;

	const paragraph = schema.nodes["paragraph"];
	const blockquote = schema.nodes["blockquote"];
	if (!paragraph || !blockquote) return null;

	const blocks = parseParagraphs(schema, quoteLines);
	const content = blocks.length > 0 ? blocks : [paragraph.create()];
	const node = blockquote.create(null, content);
	return { node, linesConsumed: index - startIndex };
};

const parseHorizontalRule = (schema: Schema, line: string): PMNode | null => {
	if (!/^[ \t]*-{3,}[ \t]*$/.exec(line)) return null;
	const hrNode = schema.nodes["horizontal_rule"];
	if (!hrNode) return null;
	return hrNode.create();
};

const blockTypes: Array<(schema: Schema, line: string) => PMNode | null> = [
	parseHeading,
	parseHorizontalRule,
];

const multiblockTypes: Array<
	(schema: Schema, lines: string[], startIndex: number) => { node: PMNode; linesConsumed: number } | null
> = [
	parseYoutubeEmbed,
	parseMathBlock,
	parseDetails,
	parseSpoiler,
	parseDefinitionList,
	parseColumns,
	parseTabs,
	parseFencedTable,
	parseAdmonitionBlock,
	parseBlockquote,
	parseMarkdownTable,
	parseList,
	parseCodeBlock,
];

export const parseZealotScript = (schema: Schema, input: string): PMNode => {
	const lines = input.replace(/\r\n/g, "\n").split("\n");
	const blocks: PMNode[] = [];
	let index = 0;

	while (index < lines.length) {
		const line = lines[index];
		if (line === undefined) { index++; continue; }

		let done = false;

		for (const t of blockTypes) {
			const node = t(schema, line);
			if (node) {
				blocks.push(node);
				index++;
				done = true;
				break;
			}
		}
		if (done) continue;

		for (const t of multiblockTypes) {
			const result = t(schema, lines, index);
			if (result) {
				blocks.push(result.node);
				index += result.linesConsumed;
				done = true;
				break;
			}
		}
		if (done) continue;

		const paragraphs = parseParagraphs(schema, [line]);
		blocks.push(...paragraphs);
		index++;
	}

	const doc = schema.nodes["doc"];
	if (!doc) throw new Error("Schema missing doc node");
	return doc.createAndFill({}, blocks)!;
};
