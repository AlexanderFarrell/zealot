import type { Node as PMNode, Schema } from "prosemirror-model";
import { parseInlineNodes } from "./parse_inline";
import { parseList } from "./parse_list";
import { parseCodeBlock } from "./parse_code_block";
import { parseParagraphs } from "./parse_paragraph";
import { parseMarkdownTable } from "./parse_table";
import { parseMathBlock } from "./parse_math_block";

const parseHeading = (schema: Schema, line: string): PMNode | null => {
	const match = /^(#{1,6})\s+(.*)$/.exec(line);
	if (!match) return null;
	const level = match[1]?.length ?? 1;
	const text = match[2] ?? "";
	const heading = schema.nodes["heading"];
	if (!heading) return null;
	return heading.create({ level }, parseInlineNodes(schema, text));
};

const parseBlockContent = (schema: Schema, lines: string[]): PMNode[] => {
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

		const math = parseMathBlock(schema, lines, index);
		if (math) { blocks.push(math.node); index += math.linesConsumed; continue; }

		const table = parseMarkdownTable(schema, lines, index);
		if (table) { blocks.push(table.node); index += table.linesConsumed; continue; }

		blocks.push(...parseParagraphs(schema, [line]));
		index++;
	}
	return blocks;
};

const extractFenceContent = (
	lines: string[],
	startIndex: number,
): { contentLines: string[]; linesConsumed: number } => {
	const contentLines: string[] = [];
	let index = startIndex + 1;
	while (index < lines.length) {
		const line = lines[index];
		if (line === undefined) break;
		if (line.trim() === ":::") { index++; break; }
		contentLines.push(line);
		index++;
	}
	return { contentLines, linesConsumed: index - startIndex };
};

const extractNestedSubBlocks = (
	lines: string[],
	startIndex: number,
	subOpenRe: RegExp,
): { subBlocks: Array<{ headerLine: string; contentLines: string[] }>; linesConsumed: number } => {
	let index = startIndex + 1;
	const subBlocks: Array<{ headerLine: string; contentLines: string[] }> = [];
	let currentHeader: string | null = null;
	let currentContent: string[] = [];
	let depth = 0;

	while (index < lines.length) {
		const line = lines[index];
		if (line === undefined) break;

		if (line.trim() === ":::") {
			if (depth === 0) {
				if (currentHeader !== null) {
					subBlocks.push({ headerLine: currentHeader, contentLines: currentContent });
				}
				index++;
				break;
			} else {
				depth--;
				if (currentHeader !== null) currentContent.push(line);
				index++;
				continue;
			}
		}

		const isSubOpen = subOpenRe.test(line);
		const isAnyFenceOpen = /^:::([A-Za-z])/.test(line);

		if (isSubOpen && depth === 0) {
			if (currentHeader !== null) {
				subBlocks.push({ headerLine: currentHeader, contentLines: currentContent });
			}
			currentHeader = line;
			currentContent = [];
			index++;
			continue;
		}

		if (isAnyFenceOpen && !isSubOpen && currentHeader !== null) {
			depth++;
		}

		if (currentHeader !== null) {
			currentContent.push(line);
		}
		index++;
	}

	return { subBlocks, linesConsumed: index - startIndex };
};

export const parseDetails = (
	schema: Schema,
	lines: string[],
	startIndex: number,
): { node: PMNode; linesConsumed: number } | null => {
	const openLine = lines[startIndex];
	if (!openLine) return null;
	const m = /^:::details(?:\s+(.+?))?\s*$/.exec(openLine);
	if (!m) return null;
	const summary = (m[1] ?? "").trim();

	const detailsNode = schema.nodes["details"];
	const paragraph = schema.nodes["paragraph"];
	if (!detailsNode || !paragraph) return null;

	const { contentLines, linesConsumed } = extractFenceContent(lines, startIndex);
	const blocks = parseBlockContent(schema, contentLines);
	const content = blocks.length > 0 ? blocks : [paragraph.create()];
	const node = detailsNode.createAndFill({ summary }, content);
	if (!node) return null;
	return { node, linesConsumed };
};

export const parseSpoiler = (
	schema: Schema,
	lines: string[],
	startIndex: number,
): { node: PMNode; linesConsumed: number } | null => {
	const openLine = lines[startIndex];
	if (!openLine) return null;
	if (!/^:::spoiler\s*$/.test(openLine)) return null;

	const spoilerNode = schema.nodes["spoiler"];
	const paragraph = schema.nodes["paragraph"];
	if (!spoilerNode || !paragraph) return null;

	const { contentLines, linesConsumed } = extractFenceContent(lines, startIndex);
	const blocks = parseBlockContent(schema, contentLines);
	const content = blocks.length > 0 ? blocks : [paragraph.create()];
	const node = spoilerNode.createAndFill({}, content);
	if (!node) return null;
	return { node, linesConsumed };
};

export const parseDefinitionList = (
	schema: Schema,
	lines: string[],
	startIndex: number,
): { node: PMNode; linesConsumed: number } | null => {
	const openLine = lines[startIndex];
	if (!openLine) return null;
	if (!/^:::definition\s*$/.test(openLine)) return null;

	const defListNode = schema.nodes["definition_list"];
	const defTermNode = schema.nodes["definition_term"];
	const defDescNode = schema.nodes["definition_desc"];
	const paragraph = schema.nodes["paragraph"];
	if (!defListNode || !defTermNode || !defDescNode || !paragraph) return null;

	const { contentLines, linesConsumed } = extractFenceContent(lines, startIndex);

	type Pair = { term: string; descs: string[] };
	const pairs: Pair[] = [];
	let currentPair: Pair | null = null;

	for (const line of contentLines) {
		if (line.trim() === "") continue;
		if (line.startsWith(": ")) {
			const descText = line.slice(2).trim();
			if (!currentPair) {
				currentPair = { term: "", descs: [] };
				pairs.push(currentPair);
			}
			currentPair.descs.push(descText);
		} else {
			currentPair = { term: line.trim(), descs: [] };
			pairs.push(currentPair);
		}
	}

	if (pairs.length === 0) return null;

	const dlChildren: PMNode[] = [];
	for (const pair of pairs) {
		const termInline = parseInlineNodes(schema, pair.term);
		const termNode = defTermNode.create(null, termInline.length > 0 ? termInline : []);
		dlChildren.push(termNode);

		const descs = pair.descs.length > 0 ? pair.descs : [""];
		for (const descText of descs) {
			const descParagraph = descText.trim().length > 0
				? paragraph.create(null, parseInlineNodes(schema, descText))
				: paragraph.create();
			const descNode = defDescNode.createAndFill({}, [descParagraph]);
			if (descNode) dlChildren.push(descNode);
		}
	}

	const node = defListNode.createAndFill({}, dlChildren);
	if (!node) return null;
	return { node, linesConsumed };
};

export const parseColumns = (
	schema: Schema,
	lines: string[],
	startIndex: number,
): { node: PMNode; linesConsumed: number } | null => {
	const openLine = lines[startIndex];
	if (!openLine) return null;
	if (!/^:::columns\s*$/.test(openLine)) return null;

	const columnsNode = schema.nodes["columns"];
	const columnNode = schema.nodes["column"];
	const paragraph = schema.nodes["paragraph"];
	if (!columnsNode || !columnNode || !paragraph) return null;

	const COL_OPEN_RE = /^:::col\s*$/;
	const { subBlocks, linesConsumed } = extractNestedSubBlocks(lines, startIndex, COL_OPEN_RE);

	if (subBlocks.length < 2) return null;

	const colNodes: PMNode[] = [];
	for (const sub of subBlocks) {
		const blocks = parseBlockContent(schema, sub.contentLines);
		const content = blocks.length > 0 ? blocks : [paragraph.create()];
		const col = columnNode.createAndFill({}, content);
		if (col) colNodes.push(col);
	}

	if (colNodes.length < 2) return null;

	const node = columnsNode.createAndFill({}, colNodes);
	if (!node) return null;
	return { node, linesConsumed };
};

export const parseTabs = (
	schema: Schema,
	lines: string[],
	startIndex: number,
): { node: PMNode; linesConsumed: number } | null => {
	const openLine = lines[startIndex];
	if (!openLine) return null;
	if (!/^:::tabs\s*$/.test(openLine)) return null;

	const tabsNode = schema.nodes["tabs"];
	const tabNode = schema.nodes["tab"];
	const paragraph = schema.nodes["paragraph"];
	if (!tabsNode || !tabNode || !paragraph) return null;

	const TAB_OPEN_RE = /^:::tab(?:\s+.+)?\s*$/;
	const { subBlocks, linesConsumed } = extractNestedSubBlocks(lines, startIndex, TAB_OPEN_RE);

	if (subBlocks.length === 0) return null;

	const tabNodes: PMNode[] = [];
	for (const sub of subBlocks) {
		const titleMatch = /^:::tab(?:\s+(.+?))?\s*$/.exec(sub.headerLine);
		const title = (titleMatch?.[1] ?? "").trim();
		const blocks = parseBlockContent(schema, sub.contentLines);
		const content = blocks.length > 0 ? blocks : [paragraph.create()];
		const t = tabNode.createAndFill({ title }, content);
		if (t) tabNodes.push(t);
	}

	if (tabNodes.length === 0) return null;

	const node = tabsNode.createAndFill({}, tabNodes);
	if (!node) return null;
	return { node, linesConsumed };
};
