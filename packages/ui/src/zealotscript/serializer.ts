import type { Mark, Node as PMNode } from "prosemirror-model";
import { escapeTableCell } from "./table_utils";

const MARK_ORDER = [
	"strong",
	"em",
	"strike",
	"underline",
	"subscript",
	"superscript",
	"highlight",
	"color",
] as const;

const MARK_DELIMITERS: Record<string, { open: string; close: string }> = {
	strong: { open: "**", close: "**" },
	em: { open: "*", close: "*" },
	strike: { open: "~~", close: "~~" },
	underline: { open: "_", close: "_" },
	subscript: { open: "<sub>", close: "</sub>" },
	superscript: { open: "<sup>", close: "</sup>" },
	highlight: { open: "<mark>", close: "</mark>" },
};

const escapeText = (text: string): string => {
	return text.replace(/\\/g, "\\\\").replace(/([*_~`<>\[\]])/g, "\\$1");
};

const escapeCodeText = (text: string): string => {
	return text.replace(/\\/g, "\\\\").replace(/`/g, "\\`");
};

const escapeColorText = (text: string): string => {
	return text.replace(/</g, "\\<").replace(/>/g, "\\>");
};

const wrapWithMarks = (text: string, marks: ReadonlyArray<Mark>): string => {
	if (text.length === 0) return text;
	const hasCodeMark = marks.some((mark) => mark.type.name === "code");
	if (hasCodeMark) return `\`${escapeCodeText(text)}\``;
	let out = escapeText(text);
	for (const markName of MARK_ORDER) {
		if (!marks.some((mark) => mark.type.name === markName)) continue;
		if (markName === "color") {
			const colorMark = marks.find((m) => m.type.name === "color");
			if (colorMark) out = `<color:${colorMark.attrs.value}>${escapeColorText(out)}</color>`;
			continue;
		}
		const delimiter = MARK_DELIMITERS[markName];
		if (!delimiter) continue;
		out = `${delimiter.open}${out}${delimiter.close}`;
	}
	return out;
};

const wrapInlineLiteralWithMarks = (content: string, marks: ReadonlyArray<Mark>): string => {
	if (content.length === 0) return content;
	let out = content;
	const nonLinkMarks = marks.filter((mark) => mark.type.name !== "link");
	const hasCodeMark = nonLinkMarks.some((mark) => mark.type.name === "code");
	const styledMarks = nonLinkMarks.filter((mark) => mark.type.name !== "code");
	for (const markName of MARK_ORDER) {
		if (!styledMarks.some((mark) => mark.type.name === markName)) continue;
		if (markName === "color") {
			const colorMark = styledMarks.find((m) => m.type.name === "color");
			if (colorMark) out = `<color:${colorMark.attrs.value}>${escapeColorText(out)}</color>`;
			continue;
		}
		const delimiter = MARK_DELIMITERS[markName];
		if (!delimiter) continue;
		out = `${delimiter.open}${out}${delimiter.close}`;
	}
	if (hasCodeMark) out = `\`${escapeCodeText(out)}\``;
	const linkMark = marks.find((mark) => mark.type.name === "link");
	if (!linkMark) return out;
	const href = (linkMark.attrs.href || "").trim();
	if (href.length === 0) return out;
	return serializeZealotHref(out, href);
};

const serializeZealotHref = (text: string, href: string): string => {
	if (href.startsWith("zealot://item/")) {
		const raw = href.slice("zealot://item/".length);
		const hashIdx = raw.indexOf("#");
		const target = hashIdx >= 0 ? raw.slice(0, hashIdx) : raw;
		const anchor = hashIdx >= 0 ? raw.slice(hashIdx + 1) : "";
		const anchorPart = anchor.length > 0 ? `#${anchor}` : "";
		const labelPart = text !== target ? `|${text}` : "";
		return `[[${target}${anchorPart}${labelPart}]]`;
	}
	if (href.startsWith("zealot://type/")) return `[[type:${href.slice("zealot://type/".length)}]]`;
	return `[${text}](${href})`;
};

const serializeTextNode = (node: PMNode): string => {
	const text = node.text || "";
	const linkMark = node.marks.find((mark) => mark.type.name === "link");
	const nonLinkMarks = node.marks.filter((mark) => mark.type.name !== "link");
	if (!linkMark && nonLinkMarks.length === 0) return text;
	const rendered = wrapWithMarks(text, nonLinkMarks);
	if (!linkMark) return rendered;
	const href = (linkMark.attrs.href || "").trim();
	if (href.length === 0) return rendered;
	return serializeZealotHref(rendered, href);
};

const serializeMathInlineNode = (node: PMNode): string => {
	return `$${node.attrs.src || ""}$`;
};

const serializeInline = (node: PMNode): string => {
	let out = "";
	node.forEach((child) => {
		if (child.isText) {
			out += serializeTextNode(child);
			return;
		}
		if (child.type.name === "hard_break") {
			out += wrapInlineLiteralWithMarks("<br>", child.marks);
			return;
		}
		if (child.type.name === "math_inline") {
			out += serializeMathInlineNode(child);
			return;
		}
		if (child.type.name === "date_ref") {
			out += serializeDateRef(child);
			return;
		}
		out += wrapInlineLiteralWithMarks(escapeText(child.textContent || ""), child.marks);
	});
	return out;
};

const joinBlocks = (blocks: string[]): string => {
	let out = "";
	let prevEmpty = false;
	for (const block of blocks) {
		const isEmpty = block.length === 0;
		if (out.length > 0) {
			out += prevEmpty || isEmpty ? "\n" : "\n\n";
		}
		out += block;
		prevEmpty = isEmpty;
	}
	return out;
};

const serializeParagraph = (node: PMNode) => serializeInline(node);

const serializeHeading = (node: PMNode) => {
	const level = Math.max(1, Math.min(6, node.attrs.level || 1));
	return `${"#".repeat(level)} ${serializeInline(node)}`;
};

const serializeCodeBlock = (node: PMNode) => {
	const content = node.textContent || "";
	const language = (node.attrs.language || "").trim();
	const fence = language.length > 0 ? `\`\`\`${language}` : "```";
	return `${fence}\n${content}\n\`\`\``;
};

const serializeBlockquote = (node: PMNode): string => {
	const innerBlocks: string[] = [];
	node.forEach((child) => innerBlocks.push(serializeBlock(child)));
	const inner = joinBlocks(innerBlocks);
	return inner
		.split("\n")
		.map((line) => (line.length > 0 ? `> ${line}` : ">"))
		.join("\n");
};

const serializeList = (node: PMNode, ordered: boolean, indentLevel = 0): string => {
	const lines: string[] = [];
	const prefixBase = "\t".repeat(indentLevel);
	let counter = 1;

	node.forEach((item) => {
		let text = "";
		const nestedLists: PMNode[] = [];

		item.forEach((child) => {
			if (child.type.name === "paragraph" && text.length === 0) {
				text = serializeInline(child);
				return;
			}
			if (child.type.name === "bullet_list" || child.type.name === "ordered_list") {
				nestedLists.push(child);
			}
		});

		const checked: boolean | null = item.attrs.checked ?? null;
		const taskPrefix = checked === null ? "" : checked ? "[x] " : "[ ] ";
		const bullet = ordered ? `${counter}. ` : "- ";
		lines.push(prefixBase + bullet + taskPrefix + text);
		counter++;

		for (const nested of nestedLists) {
			const nestedText = serializeList(nested, nested.type.name === "ordered_list", indentLevel + 1);
			if (nestedText.trim().length > 0) lines.push(nestedText);
		}
	});

	return lines.join("\n");
};

const serializeTableCell = (cell: PMNode): string => {
	const parts: string[] = [];
	cell.forEach((child) => {
		if (child.type.name === "paragraph") {
			parts.push(serializeInline(child));
		} else {
			parts.push((child.textContent || "").replace(/\n+/g, " ").trim());
		}
	});
	return escapeTableCell(parts.join(" <br> "));
};

const serializeTable = (node: PMNode): string => {
	if (node.childCount === 0) return "";
	const rows: string[][] = [];
	const headerFlags: boolean[] = [];

	node.forEach((row) => {
		const cells: string[] = [];
		let rowIsHeader = row.childCount > 0;
		row.forEach((cell) => {
			cells.push(serializeTableCell(cell));
			if (cell.type.name !== "table_header") rowIsHeader = false;
		});
		rows.push(cells);
		headerFlags.push(rowIsHeader);
	});

	if (rows.length === 0) return "";

	if (!headerFlags[0]) {
		const lines = [":::table"];
		for (const cells of rows) lines.push(cells.join(" | "));
		lines.push(":::");
		return lines.join("\n");
	}

	const firstRow = rows[0];
	if (!firstRow) return "";
	const columnCount = firstRow.length;
	const lines: string[] = [];
	lines.push(`| ${firstRow.join(" | ")} |`);
	lines.push(`| ${Array.from({ length: columnCount }, () => "---").join(" | ")} |`);
	for (let i = 1; i < rows.length; i++) {
		const row = rows[i];
		if (row) lines.push(`| ${row.join(" | ")} |`);
	}
	return lines.join("\n");
};

const serializeYoutubeEmbed = (node: PMNode): string => {
	return `:::youtube ${(node.attrs.videoId || "").trim()}`;
};

const serializeMathBlock = (node: PMNode): string => {
	const src = (node.attrs.src || "").trim();
	return `:::math\n${src}\n:::`;
};

const serializeAdmonition = (node: PMNode): string => {
	const kind = node.attrs.kind || "note";
	const blocks: string[] = [];
	node.forEach((child) => blocks.push(serializeBlock(child)));
	const content = joinBlocks(blocks);
	if (content.length === 0) return `:::${kind}\n:::`;
	return `:::${kind}\n${content}\n:::`;
};

const serializeDateRef = (node: PMNode): string => {
	return `@${node.attrs.raw || ""}`;
};

const serializeDetails = (node: PMNode): string => {
	const summary = (node.attrs.summary as string) || "";
	const blocks: string[] = [];
	node.forEach((child) => blocks.push(serializeBlock(child)));
	const content = joinBlocks(blocks);
	const header = summary.length > 0 ? `:::details ${summary}` : ":::details";
	if (content.length === 0) return `${header}\n:::`;
	return `${header}\n${content}\n:::`;
};

const serializeSpoiler = (node: PMNode): string => {
	const blocks: string[] = [];
	node.forEach((child) => blocks.push(serializeBlock(child)));
	const content = joinBlocks(blocks);
	if (content.length === 0) return ":::spoiler\n:::";
	return `:::spoiler\n${content}\n:::`;
};

const serializeDefinitionList = (node: PMNode): string => {
	const lines: string[] = [":::definition"];
	node.forEach((child) => {
		if (child.type.name === "definition_term") {
			lines.push(serializeInline(child));
		} else if (child.type.name === "definition_desc") {
			child.forEach((block) => {
				const text = serializeBlock(block);
				const firstLine = text.split("\n")[0] ?? "";
				lines.push(`: ${firstLine}`);
			});
		}
	});
	lines.push(":::");
	return lines.join("\n");
};

const serializeColumns = (node: PMNode): string => {
	const lines: string[] = [":::columns"];
	node.forEach((col) => {
		lines.push(":::col");
		const colBlocks: string[] = [];
		col.forEach((child) => colBlocks.push(serializeBlock(child)));
		if (colBlocks.length > 0) lines.push(joinBlocks(colBlocks));
		lines.push(":::");
	});
	lines.push(":::");
	return lines.join("\n");
};

const serializeTabs = (node: PMNode): string => {
	const lines: string[] = [":::tabs"];
	node.forEach((tab) => {
		const title = (tab.attrs.title as string) || "";
		lines.push(title.length > 0 ? `:::tab ${title}` : ":::tab");
		const tabBlocks: string[] = [];
		tab.forEach((child) => tabBlocks.push(serializeBlock(child)));
		if (tabBlocks.length > 0) lines.push(joinBlocks(tabBlocks));
		lines.push(":::");
	});
	lines.push(":::");
	return lines.join("\n");
};

const nodeSerializers: Record<string, (node: PMNode) => string> = {
	paragraph: serializeParagraph,
	heading: serializeHeading,
	horizontal_rule: () => "---",
	bullet_list: (node) => serializeList(node, false),
	ordered_list: (node) => serializeList(node, true),
	code_block: serializeCodeBlock,
	blockquote: serializeBlockquote,
	table: serializeTable,
	admonition: serializeAdmonition,
	youtube_embed: serializeYoutubeEmbed,
	math_block: serializeMathBlock,
	math_inline: serializeMathInlineNode,
	date_ref: serializeDateRef,
	details: serializeDetails,
	spoiler: serializeSpoiler,
	definition_list: serializeDefinitionList,
	columns: serializeColumns,
	tabs: serializeTabs,
};

const serializeBlock = (node: PMNode): string => {
	const serializer = nodeSerializers[node.type.name];
	if (!serializer) return node.textContent || "";
	return serializer(node);
};

export const serializeZealotScript = (doc: PMNode): string => {
	const blocks: string[] = [];
	doc.forEach((node) => blocks.push(serializeBlock(node)));
	return joinBlocks(blocks);
};
