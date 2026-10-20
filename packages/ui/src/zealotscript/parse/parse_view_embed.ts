import type { Node as PMNode, Schema } from "prosemirror-model";

/** A line that is only `![[Title]]`, `![[Title|cards]]` or `![[Title|table]]` embeds a Saved View. */
export const VIEW_EMBED_RE = /^[ \t]*!\[\[([^\]|#]+?)(?:\|(cards|table))?\]\][ \t]*$/;

export const parseViewEmbed = (schema: Schema, line: string): PMNode | null => {
	const match = VIEW_EMBED_RE.exec(line);
	if (!match) return null;
	const embed = schema.nodes["view_embed"];
	if (!embed) return null;
	return embed.create({ title: (match[1] ?? "").trim(), display: match[2] ?? "" });
};
