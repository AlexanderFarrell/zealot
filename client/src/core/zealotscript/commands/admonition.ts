import type { ZCommandBlockInfo } from "./types";

const parseAdmonitionBlock: ZCommandBlockInfo["parse"] = (schema, lines, startIndex, args) => {
	const kind = args[0] || "note";
	const contentLines: string[] = [];
	let index = startIndex;

	while (index < lines.length) {
		const line = lines[index];
		if (line.trim() === ":::") break;
		contentLines.push(line);
		index++;
	}

	const text = contentLines.join("\n").trim();
	const paragraph = schema.nodes.paragraph.createAndFill({}, schema.text(text));
	const admonition = schema.nodes.admonition.createAndFill({kind}, paragraph ? [paragraph] : []);
	if (!admonition) return null;

	return {node: admonition, linesConsumed: index - startIndex + 1};
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
