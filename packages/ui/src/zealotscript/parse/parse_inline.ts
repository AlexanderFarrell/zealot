import type { Node as PMNode, Schema } from "prosemirror-model";
import { lookupEmoji } from "../emoji_map";

type InlineAtomMatch = {
	type: "mdlink" | "hard_break" | "wikilink";
	full: string;
	label: string;
	href: string;
	anchor?: string | undefined;
};

type ParseInlineRangeResult = {
	nodes: PMNode[];
	index: number;
	closed: boolean;
};

type PairedTagMark = {
	open: string;
	close: string;
	markName: string;
};

const COLOR_TAG_RE = /^<color:([^>]+)>([\s\S]*?)<\/color>/;

type SymmetricMark = {
	token: string;
	markName: string;
	requireWordBoundary: boolean;
};

const MDLINK_RE = /^\[([^\]]+)\]\(([^)]+)\)/;
const HARD_BREAK_RE = /^<br\s*\/?>/i;
const WIKILINK_RE = /^\[\[([^\]#|]+)(?:#([^\]|]+))?(?:\|([^\]]+))?\]\]/;
const WORD_CHAR_RE = /[A-Za-z0-9]/;
const DATE_DAY_RE = /^@(\d{4}-\d{2}-\d{2})\b/;
const DATE_WEEK_RE = /^@(\d{4}-W\d{2})\b/;
const DATE_YEAR_RE = /^@(\d{4})\b/;

const PAIRED_TAG_MARKS: PairedTagMark[] = [
	{ open: "<mark>", close: "</mark>", markName: "highlight" },
	{ open: "<sub>", close: "</sub>", markName: "subscript" },
	{ open: "<sup>", close: "</sup>", markName: "superscript" },
	{ open: "<u>", close: "</u>", markName: "underline" },
];

const SYMMETRIC_MARKS: SymmetricMark[] = [
	{ token: "**", markName: "strong", requireWordBoundary: false },
	{ token: "~~", markName: "strike", requireWordBoundary: false },
	{ token: "*", markName: "em", requireWordBoundary: false },
	{ token: "_", markName: "underline", requireWordBoundary: true },
];

const isWordChar = (char: string | undefined): boolean => {
	if (!char) return false;
	return WORD_CHAR_RE.test(char);
};

const hasClosingToken = (text: string, index: number, token: string): boolean => {
	let cursor = index + token.length;
	while (cursor < text.length) {
		if (text[cursor] === "\\") { cursor += 2; continue; }
		if (text.startsWith(token, cursor)) return true;
		cursor++;
	}
	return false;
};

const canOpenSymmetricMark = (text: string, index: number, token: SymmetricMark): boolean => {
	const after = text[index + token.token.length];
	if (!after) return false;
	if (!hasClosingToken(text, index, token.token)) return false;
	if (!token.requireWordBoundary) return true;
	const before = index > 0 ? text[index - 1] : undefined;
	return !isWordChar(before) && !isWordChar(after);
};

const matchInlineAtomAt = (text: string, index: number): InlineAtomMatch | null => {
	const rest = text.slice(index);

	const wikilinkMatch = WIKILINK_RE.exec(rest);
	if (wikilinkMatch) {
		const content = wikilinkMatch[1] ?? "";
		const anchor = wikilinkMatch[2];
		const displayLabel = wikilinkMatch[3];
		const isType = content.startsWith("type:");
		const target = isType ? content.slice("type:".length) : content;
		const baseHref = isType ? `zealot://type/${target}` : `zealot://item/${target}`;
		const href = anchor ? `${baseHref}#${anchor}` : baseHref;
		const label = displayLabel ?? target;
		return { type: "wikilink", full: wikilinkMatch[0], label, href, anchor };
	}

	const markdownLinkMatch = MDLINK_RE.exec(rest);
	if (markdownLinkMatch) {
		return {
			type: "mdlink",
			full: markdownLinkMatch[0],
			label: markdownLinkMatch[1] ?? "",
			href: markdownLinkMatch[2] ?? "",
		};
	}

	const hardBreakMatch = HARD_BREAK_RE.exec(rest);
	if (hardBreakMatch) {
		return { type: "hard_break", full: hardBreakMatch[0], label: "", href: "" };
	}

	return null;
};

const addMarkToNodes = (schema: Schema, nodes: PMNode[], markName: string): PMNode[] => {
	const markType = schema.marks[markName];
	if (!markType) return nodes;
	const mark = markType.create();
	return nodes.map((node) => {
		if (!node.isText) {
			if (!node.type.allowsMarkType(markType)) return node;
			if (node.marks.some((existing) => existing.eq(mark))) return node;
			return node.mark([...node.marks, mark]);
		}
		if (node.marks.some((existing) => existing.eq(mark))) return node;
		return schema.text(node.text || "", [...node.marks, mark]);
	});
};

const buildInlineAtomNodes = (schema: Schema, atom: InlineAtomMatch): PMNode[] => {
	if (atom.type === "hard_break") {
		const hardBreak = schema.nodes["hard_break"];
		if (hardBreak) return [hardBreak.create()];
		return [schema.text(atom.full)];
	}

	// mdlink or wikilink
	const href = atom.href.trim();
	const label = atom.label.trim();
	const content = label.length > 0 ? parseInlineNodes(schema, label) : [schema.text(href)];
	const linkMarkType = schema.marks["link"];
	if (!linkMarkType || href.length === 0) return content;
	const mark = linkMarkType.create({ href });
	return content.map((node) => {
		if (!node.isText) {
			if (!node.type.allowsMarkType(linkMarkType)) return node;
			if (node.marks.some((existing) => existing.eq(mark))) return node;
			return node.mark([...node.marks, mark]);
		}
		if (node.marks.some((existing) => existing.eq(mark))) return node;
		return schema.text(node.text || "", [...node.marks, mark]);
	});
};

const parseInlineRange = (
	schema: Schema,
	text: string,
	startIndex: number,
	stopToken?: string
): ParseInlineRangeResult => {
	const nodes: PMNode[] = [];
	let buffer = "";
	let index = startIndex;

	const flushBuffer = () => {
		if (buffer.length > 0) { nodes.push(schema.text(buffer)); buffer = ""; }
	};

	while (index < text.length) {
		if (stopToken && text.startsWith(stopToken, index)) {
			flushBuffer();
			return { nodes, index: index + stopToken.length, closed: true };
		}

		if (text[index] === "\\" && index + 1 < text.length) {
			buffer += text[index + 1];
			index += 2;
			continue;
		}

		if (text[index] === "@") {
			const rest = text.slice(index);
			const dayMatch = DATE_DAY_RE.exec(rest);
			const weekMatch = DATE_WEEK_RE.exec(rest);
			const yearMatch = DATE_YEAR_RE.exec(rest);
			const match = dayMatch ?? weekMatch ?? yearMatch;
			if (match) {
				const raw = match[1] ?? "";
				const kind = dayMatch ? "day" : weekMatch ? "week" : "year";
				const dateRefNode = schema.nodes["date_ref"];
				if (dateRefNode) {
					flushBuffer();
					nodes.push(dateRefNode.create({ raw, kind }));
					index += match[0].length;
					continue;
				}
			}
		}

		const atom = matchInlineAtomAt(text, index);
		if (atom) {
			flushBuffer();
			nodes.push(...buildInlineAtomNodes(schema, atom));
			index += atom.full.length;
			continue;
		}

		const pairedTag = PAIRED_TAG_MARKS.find((token) => text.startsWith(token.open, index));
		if (pairedTag) {
			flushBuffer();
			const inner = parseInlineRange(schema, text, index + pairedTag.open.length, pairedTag.close);
			if (inner.closed && inner.nodes.length > 0) {
				nodes.push(...addMarkToNodes(schema, inner.nodes, pairedTag.markName));
				index = inner.index;
			} else {
				buffer += pairedTag.open;
				index += pairedTag.open.length;
			}
			continue;
		}

		if (text[index] === "$" && text[index + 1] !== " " && text[index + 1] !== undefined) {
			const closeIndex = text.indexOf("$", index + 1);
			if (closeIndex > index + 1 && text[closeIndex - 1] !== " ") {
				const mathSrc = text.slice(index + 1, closeIndex);
				const mathInline = schema.nodes["math_inline"];
				if (mathInline) {
					flushBuffer();
					nodes.push(mathInline.create({ src: mathSrc }));
					index = closeIndex + 1;
					continue;
				}
			}
		}

		if (text[index] === ":" && index + 2 < text.length) {
			const closeIndex = text.indexOf(":", index + 1);
			if (closeIndex > index + 1) {
				const shortcode = text.slice(index + 1, closeIndex);
				if (/^[a-z0-9_+\-]+$/.test(shortcode)) {
					const emojiChar = lookupEmoji(shortcode);
					if (emojiChar) {
						flushBuffer();
						nodes.push(schema.text(emojiChar));
						index = closeIndex + 1;
						continue;
					}
				}
			}
		}

		if (text[index] === "`") {
			const closeIndex = text.indexOf("`", index + 1);
			if (closeIndex > index + 1) {
				flushBuffer();
				const codeText = text.slice(index + 1, closeIndex);
				const codeMark = schema.marks["code"]?.create();
				if (codeMark) {
					nodes.push(schema.text(codeText, [codeMark]));
				} else {
					nodes.push(schema.text(codeText));
				}
				index = closeIndex + 1;
				continue;
			}
		}

		if (text[index] === "<") {
			const colorMatch = COLOR_TAG_RE.exec(text.slice(index));
			if (colorMatch) {
				const colorValue = colorMatch[1] ?? "";
				const innerText = colorMatch[2] ?? "";
				const colorMarkType = schema.marks["color"];
				if (colorMarkType) {
					flushBuffer();
					const innerNodes = parseInlineNodes(schema, innerText);
					const mark = colorMarkType.create({ value: colorValue });
					nodes.push(...innerNodes.map((n) => {
						if (n.isText) return schema.text(n.text || "", [...n.marks, mark]);
						if (!n.type.allowsMarkType(colorMarkType)) return n;
						return n.mark([...n.marks, mark]);
					}));
					index += colorMatch[0].length;
					continue;
				}
			}
		}

		let consumedSymmetricMark = false;
		for (const token of SYMMETRIC_MARKS) {
			if (!text.startsWith(token.token, index)) continue;
			if (!canOpenSymmetricMark(text, index, token)) continue;
			flushBuffer();
			const inner = parseInlineRange(schema, text, index + token.token.length, token.token);
			if (inner.closed && inner.nodes.length > 0) {
				nodes.push(...addMarkToNodes(schema, inner.nodes, token.markName));
				index = inner.index;
				consumedSymmetricMark = true;
				break;
			}
		}
		if (consumedSymmetricMark) continue;

		buffer += text[index];
		index++;
	}

	flushBuffer();
	return { nodes, index, closed: false };
};

export const parseInlineNodes = (schema: Schema, text: string): PMNode[] => {
	return parseInlineRange(schema, text, 0).nodes;
};

export const parseInline = (schema: Schema, text: string): PMNode => {
	const content = parseInlineNodes(schema, text);
	const paragraph = schema.nodes["paragraph"];
	if (!paragraph) throw new Error("Schema missing paragraph node");
	return paragraph.create(null, content);
};
