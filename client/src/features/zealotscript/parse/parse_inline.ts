import type {Node as PMNode, Schema} from "prosemirror-model";

type InlineAtomMatch = {
	type: "itemlink" | "mdlink" | "cmdlink" | "image" | "hard_break" | "citation";
	full: string;
	label?: string;
	href?: string;
	ref?: string;
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

type SymmetricMark = {
	token: string;
	markName: string;
	requireWordBoundary: boolean;
};

const IMAGE_RE = /^!\[([^\]]*)\]\(([^)]+)\)/;
const ITEMLINK_RE = /^\[\[([^\]]+)\]\]/;
const MDLINK_RE = /^\[([^\]]+)\]\(([^)]+)\)/;
const CMDLINK_RE = /^:::link\|([^|\n]*)\|([^|\n]+)/;
const HARD_BREAK_RE = /^<br\s*\/?>/i;
const CITATION_RE = /^cite([^\n]+)/;
const WORD_CHAR_RE = /[A-Za-z0-9]/;

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
}

const hasClosingToken = (text: string, index: number, token: string): boolean => {
	let cursor = index + token.length;
	while (cursor < text.length) {
		if (text[cursor] === "\\") {
			cursor += 2;
			continue;
		}
		if (text.startsWith(token, cursor)) return true;
		cursor++;
	}
	return false;
}

const canOpenSymmetricMark = (text: string, index: number, token: SymmetricMark): boolean => {
	const after = text[index + token.token.length];
	if (!after) return false;
	if (!hasClosingToken(text, index, token.token)) return false;
	if (!token.requireWordBoundary) return true;
	const before = index > 0 ? text[index - 1] : undefined;
	return !isWordChar(before) && !isWordChar(after);
}

const matchInlineAtomAt = (text: string, index: number): InlineAtomMatch | null => {
	const rest = text.slice(index);

	const imageMatch = IMAGE_RE.exec(rest);
	if (imageMatch) {
		return {
			type: "image",
			full: imageMatch[0],
			label: imageMatch[1],
			href: imageMatch[2]
		};
	}

	const itemLinkMatch = ITEMLINK_RE.exec(rest);
	if (itemLinkMatch) {
		return {
			type: "itemlink",
			full: itemLinkMatch[0],
			label: itemLinkMatch[1]
		};
	}

	const markdownLinkMatch = MDLINK_RE.exec(rest);
	if (markdownLinkMatch) {
		return {
			type: "mdlink",
			full: markdownLinkMatch[0],
			label: markdownLinkMatch[1],
			href: markdownLinkMatch[2]
		};
	}

	const commandLinkMatch = CMDLINK_RE.exec(rest);
	if (commandLinkMatch) {
		return {
			type: "cmdlink",
			full: commandLinkMatch[0],
			label: commandLinkMatch[1],
			href: commandLinkMatch[2]
		};
	}

	const hardBreakMatch = HARD_BREAK_RE.exec(rest);
	if (hardBreakMatch) {
		return {
			type: "hard_break",
			full: hardBreakMatch[0]
		};
	}

	const citationMatch = CITATION_RE.exec(rest);
	if (citationMatch) {
		return {
			type: "citation",
			full: citationMatch[0],
			ref: citationMatch[1]
		};
	}

	return null;
}

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
}

const buildInlineAtomNodes = (schema: Schema, atom: InlineAtomMatch): PMNode[] => {
	if (atom.type === "itemlink") {
		const title = (atom.label || "").trim();
		const href = `/item/${title}`;
		return [schema.nodes.itemlink.create({ title, href })];
	}

	if (atom.type === "image") {
		const src = (atom.href || "").trim();
		const alt = (atom.label || "").trim();
		const imageNode = schema.nodes.image;
		if (imageNode && src.length > 0) {
			return [imageNode.create({ src, alt })];
		}
		return [schema.text(atom.full)];
	}

	if (atom.type === "hard_break") {
		const hardBreak = schema.nodes.hard_break;
		if (hardBreak) {
			return [hardBreak.create()];
		}
		return [schema.text(atom.full)];
	}

	if (atom.type === "citation") {
		const citationNode = schema.nodes.citation;
		const ref = (atom.ref || "").trim();
		if (citationNode && ref.length > 0) {
			return [citationNode.create({ ref })];
		}
		return [schema.text(atom.full)];
	}

	const href = (atom.href || "").trim();
	const label = (atom.label || "").trim();
	const content = label.length > 0 ? parseInlineNodes(schema, label) : [schema.text(href)];
	const linkMarkType = schema.marks.link;
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
}

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
		if (buffer.length > 0) {
			nodes.push(schema.text(buffer));
			buffer = "";
		}
	}

	while (index < text.length) {
		if (stopToken && text.startsWith(stopToken, index)) {
			flushBuffer();
			return {
				nodes,
				index: index + stopToken.length,
				closed: true
			};
		}

		if (text[index] === "\\" && index + 1 < text.length) {
			buffer += text[index + 1];
			index += 2;
			continue;
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

		if (text[index] === "`") {
			const closeIndex = text.indexOf("`", index + 1);
			if (closeIndex > index + 1) {
				flushBuffer();
				const codeText = text.slice(index + 1, closeIndex);
				const codeMark = schema.marks.code?.create();
				if (codeMark) {
					nodes.push(schema.text(codeText, [codeMark]));
				} else {
					nodes.push(schema.text(codeText));
				}
				index = closeIndex + 1;
				continue;
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
	return {
		nodes,
		index,
		closed: false
	};
}

export const parseInlineNodes = (schema: Schema, text: string): PMNode[] => {
	const parsed = parseInlineRange(schema, text, 0);
	return parsed.nodes;
}

export const parseInline = (schema: Schema, text: string): PMNode => {
	const content = parseInlineNodes(schema, text);
	return schema.nodes.paragraph.create(null, content);
}
