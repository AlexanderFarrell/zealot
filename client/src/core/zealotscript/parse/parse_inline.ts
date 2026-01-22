import type {Node as PMNode, Schema} from "prosemirror-model";

export type InlineToken = {text: string};


type InlineMatch = {
	type: "itemlink" | "mdlink" | "cmdlink";
	index: number;
	full: string;
	label?: string;
	href?: string;
}

const findNextMatch = (text: string, from: number): InlineMatch | null => {
	const patterns = [
		{type: "itemlink" as const, re: /\[\[([^\]]+)\]\]/g},
		{type: "mdlink" as const, re: /\[([^\]]+)\]\(([^)]+)\)/g},
		{type: "cmdlink" as const, re: /:::link\|([^|\n]*)\|([^|\n]+)/g}
	];

	let best: InlineMatch | null = null;

	for (const pattern of patterns) {
		pattern.re.lastIndex = from;
		const match = pattern.re.exec(text);
		if (!match) continue;

		const candidate: InlineMatch = {
			type: pattern.type,
			index: match.index,
			full: match[0],
			label: match[1],
			href: match[2]
		};

		if (!best || candidate.index < best.index) {
			best = candidate;
		}
	}

	return best;
}

const parseInlineContent = (schema: Schema, text: string): PMNode[] => {
	const nodes: PMNode[] = [];
	let index = 0;

	while (index < text.length) {
		const match = findNextMatch(text, index);
		if (!match) break;

		if (match.index > index) {
			const raw = text.slice(index, match.index);
			if (raw.length > 0) nodes.push(schema.text(raw));
		}

		if (match.type == "itemlink") {
			const title = (match.label || "").trim();
			const href = `/item/${title}`;
			nodes.push(schema.nodes.itemlink.create({title, href}));
		} else {
			const href = (match.href || "").trim();
			const label = (match.label || "").trim();
			const textLabel = label.length > 0 ? label : href;
			const mark = schema.marks.link?.create({href});
			if (mark) {
				nodes.push(schema.text(textLabel, [mark]));
			} else {
				nodes.push(schema.text(textLabel));
			}
		}

		index = match.index + match.full.length;
	}

	if (index < text.length) {
		const rest = text.slice(index);
		if (rest.length > 0) nodes.push(schema.text(rest));
	}

	return nodes;
}

export const parseInline = (schema: Schema, text: string): PMNode => {
	const content = parseInlineContent(schema, text);
	return schema.nodes.paragraph.create(null, content);
}