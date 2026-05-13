import type { Node as PMNode, Schema } from "prosemirror-model";

export const extractYouTubeVideoId = (input: string): string => {
	const trimmed = input.trim();
	const watchMatch = /[?&]v=([A-Za-z0-9_-]{11})/.exec(trimmed);
	if (watchMatch?.[1]) return watchMatch[1];
	const shortMatch = /youtu\.be\/([A-Za-z0-9_-]{11})/.exec(trimmed);
	if (shortMatch?.[1]) return shortMatch[1];
	const embedMatch = /youtube\.com\/embed\/([A-Za-z0-9_-]{11})/.exec(trimmed);
	if (embedMatch?.[1]) return embedMatch[1];
	return trimmed;
};

export const parseYoutubeEmbed = (schema: Schema, lines: string[], startIndex: number): { node: PMNode; linesConsumed: number } | null => {
	const line = lines[startIndex];
	if (!line) return null;
	const match = /^:::youtube\s+(.+)$/.exec(line);
	if (!match) return null;
	const videoId = extractYouTubeVideoId(match[1] ?? "");
	const youtubeEmbed = schema.nodes["youtube_embed"];
	if (!youtubeEmbed) return null;
	return { node: youtubeEmbed.create({ videoId }), linesConsumed: 1 };
};
