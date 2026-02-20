import type {Mark, Node as PMNode} from "prosemirror-model";
import { escapeTableCell } from "./table_utils";

const MARK_ORDER = [
	"strong",
	"em",
	"strike",
	"underline",
	"subscript",
	"superscript",
	"highlight"
] as const;

const MARK_DELIMITERS: Record<string, {open: string, close: string}> = {
	strong: { open: "**", close: "**" },
	em: { open: "*", close: "*" },
	strike: { open: "~~", close: "~~" },
	underline: { open: "_", close: "_" },
	subscript: { open: "<sub>", close: "</sub>" },
	superscript: { open: "<sup>", close: "</sup>" },
	highlight: { open: "<mark>", close: "</mark>" }
};

const escapeText = (text: string): string => {
	return text
		.replace(/\\/g, "\\\\")
		.replace(/([*_~`<>\[\]])/g, "\\$1");
}

const escapeCodeText = (text: string): string => {
	return text.replace(/\\/g, "\\\\").replace(/`/g, "\\`");
}

const wrapWithMarks = (text: string, marks: ReadonlyArray<Mark>): string => {
	if (text.length === 0) return text;

	const hasCodeMark = marks.some((mark) => mark.type.name === "code");
	if (hasCodeMark) {
		return `\`${escapeCodeText(text)}\``;
	}

	let out = escapeText(text);
	for (const markName of MARK_ORDER) {
		if (!marks.some((mark) => mark.type.name === markName)) continue;
		const delimiter = MARK_DELIMITERS[markName];
		out = `${delimiter.open}${out}${delimiter.close}`;
	}
	return out;
}

const wrapInlineLiteralWithMarks = (content: string, marks: ReadonlyArray<Mark>): string => {
	if (content.length === 0) return content;

	let out = content;
	const nonLinkMarks = marks.filter((mark) => mark.type.name !== "link");
	const hasCodeMark = nonLinkMarks.some((mark) => mark.type.name === "code");
	const styledMarks = nonLinkMarks.filter((mark) => mark.type.name !== "code");

	for (const markName of MARK_ORDER) {
		if (!styledMarks.some((mark) => mark.type.name === markName)) continue;
		const delimiter = MARK_DELIMITERS[markName];
		out = `${delimiter.open}${out}${delimiter.close}`;
	}

	if (hasCodeMark) {
		out = `\`${escapeCodeText(out)}\``;
	}

	const linkMark = marks.find((mark) => mark.type.name === "link");
	if (!linkMark) return out;
	const href = (linkMark.attrs.href || "").trim();
	if (href.length === 0) return out;
	return `[${out}](${href})`;
}

const serializeTextNode = (node: PMNode): string => {
	const text = node.text || "";
	const linkMark = node.marks.find((mark) => mark.type.name === "link");
	const nonLinkMarks = node.marks.filter((mark) => mark.type.name !== "link");
	if (!linkMark && nonLinkMarks.length === 0) {
		return text;
	}
	const rendered = wrapWithMarks(text, nonLinkMarks);

	if (!linkMark) return rendered;
	const href = (linkMark.attrs.href || "").trim();
	if (href.length === 0) return rendered;
	return `[${rendered}](${href})`;
}

const serializeInline = (node: PMNode) => {
	let out = "";
	node.forEach((child) => {
		if (child.isText) {
			out += serializeTextNode(child);
			return;
		}

		if (child.type.name === "itemlink") {
			out += wrapInlineLiteralWithMarks(`[[${child.attrs.title || ""}]]`, child.marks);
			return;
		}

		if (child.type.name === "image") {
			const src = child.attrs.src || "";
			const alt = child.attrs.alt || "";
			if (src.length > 0) {
				out += wrapInlineLiteralWithMarks(`![${alt}](${src})`, child.marks);
			}
			return;
		}

		if (child.type.name === "hard_break") {
			out += wrapInlineLiteralWithMarks("<br>", child.marks);
			return;
		}

		if (child.type.name === "citation") {
			const ref = (child.attrs.ref || "").trim();
			if (ref.length > 0) {
				out += wrapInlineLiteralWithMarks(`cite${ref}`, child.marks);
			}
			return;
		}

		out += wrapInlineLiteralWithMarks(escapeText(child.textContent || ""), child.marks);
	});
	return out;
}

const joinBlocks = (blocks: string[]): string => {
	let out = "";
	let prevEmpty = false;

	for (const block of blocks) {
		const isEmpty = block.length === 0;
		if (out.length > 0) {
			out += (prevEmpty || isEmpty) ? "\n" : "\n\n";
		}
		out += block;
		prevEmpty = isEmpty;
	}

	return out;
}

const serializeParagraph = (node: PMNode) => {
	return serializeInline(node);
}

const serializeHeading = (node: PMNode) => {
	const level = Math.max(1, Math.min(6, node.attrs.level || 1));
	return `${"#".repeat(level)} ${serializeInline(node)}`;
}

const serializeCodeBlock = (node: PMNode) => {
	const content = node.textContent || "";
	return `\`\`\`\n${content}\n\`\`\``;
}

const serializeBlockquote = (node: PMNode): string => {
	const innerBlocks: string[] = [];
	node.forEach((child) => {
		innerBlocks.push(serializeBlock(child));
	});

	const inner = joinBlocks(innerBlocks);
	return inner
		.split("\n")
		.map((line) => (line.length > 0 ? `> ${line}` : ">"))
		.join("\n");
}

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

		const bullet = ordered ? `${counter}. ` : "- ";
		lines.push(prefixBase + bullet + text);
		counter++;

		for (const nested of nestedLists) {
			const nestedOrdered = nested.type.name === "ordered_list";
			const nestedText = serializeList(nested, nestedOrdered, indentLevel + 1);
			if (nestedText.trim().length > 0) {
				lines.push(nestedText);
			}
		}
	});

	return lines.join("\n");
}

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
}

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

	const hasHeaderRow = headerFlags[0];
	if (!hasHeaderRow) {
		const lines: string[] = [":::table"];
		for (const cells of rows) {
			lines.push(cells.join(" | "));
		}
		lines.push(":::");
		return lines.join("\n");
	}

	const columnCount = rows[0].length;
	const lines: string[] = [];
	lines.push(`| ${rows[0].join(" | ")} |`);
	lines.push(`| ${Array.from({ length: columnCount }, () => "---").join(" | ")} |`);

	for (let rowIndex = 1; rowIndex < rows.length; rowIndex++) {
		lines.push(`| ${rows[rowIndex].join(" | ")} |`);
	}

	return lines.join("\n");
}

const serializeAdmonition = (node: PMNode): string => {
	const kind = node.attrs.kind || "note";
	const blocks: string[] = [];
	node.forEach((child) => {
		blocks.push(serializeBlock(child));
	});
	const content = joinBlocks(blocks);
	if (content.length === 0) {
		return `:::${kind}\n:::`;
	}
	return `:::${kind}\n${content}\n:::`;
}

const nodeSerializers: Record<string, (node: PMNode) => string> = {
	paragraph: serializeParagraph,
	heading: serializeHeading,
	bullet_list: (node) => serializeList(node, false),
	ordered_list: (node) => serializeList(node, true),
	code_block: serializeCodeBlock,
	blockquote: serializeBlockquote,
	table: serializeTable,
	admonition: serializeAdmonition
};

const serializeBlock = (node: PMNode): string => {
	const kind = node.type.name;
	const serializer = nodeSerializers[kind];
	if (!serializer) return node.textContent || "";
	return serializer(node);
}

export const serializeZealotScript = (doc: PMNode): string => {
	const blocks: string[] = [];
	doc.forEach((node) => {
		blocks.push(serializeBlock(node));
	});
	return joinBlocks(blocks);
}
