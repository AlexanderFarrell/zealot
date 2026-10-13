// Normalises Markdown copied from other tools (GitHub, Notion, ChatGPT, Obsidian, Word
// exports) into the dialect parseZealotScript reads. This runs on paste only: stored
// ZealotScript is never rewritten, so existing documents round-trip unchanged.

const FENCE_OPEN_RE = /^([ \t]*)(```|~~~)(.*)$/;
const LIST_LINE_RE = /^[ \t]*(?:[-*+]|\d+[.)])(?:[ \t]+|$)/;
const HR_RE = /^[ \t]{0,3}(?:(?:\*[ \t]*){3,}|(?:_[ \t]*){3,}|(?:-[ \t]*){3,})$/;
const SETEXT_RE = /^[ \t]{0,3}(=+|-+)[ \t]*$/;
const ENTITIES: Record<string, string> = {
	"&amp;": "&", "&lt;": "<", "&gt;": ">", "&quot;": '"', "&#39;": "'", "&nbsp;": " ",
};

const decodeEntities = (line: string): string =>
	line.replace(/&(?:amp|lt|gt|quot|#39|nbsp);/g, (m) => ENTITIES[m] ?? m);

const isBlank = (line: string | undefined): boolean => (line ?? "").trim().length === 0;

export const importMarkdown = (input: string): string => {
	const source = input
		.replace(/\r\n?/g, "\n")
		.replace(/[\u200B-\u200D\uFEFF]/g, "")
		.replace(/\u00A0/g, " ")
		.split("\n");

	// Pass 1: line-local rewrites, leaving fenced code untouched.
	const lines: string[] = [];
	let fence: string | null = null;
	for (const raw of source) {
		const open = FENCE_OPEN_RE.exec(raw);
		if (fence) {
			if (open && open[2] === fence && (open[3] ?? "").trim() === "") {
				lines.push(`${open[1] ?? ""}\`\`\``);
				fence = null;
			} else {
				lines.push(raw);
			}
			continue;
		}
		if (open) {
			fence = open[2] ?? "```";
			lines.push(`${open[1] ?? ""}\`\`\`${(open[3] ?? "").trim()}`);
			continue;
		}
		lines.push(HR_RE.test(raw) ? "---" : decodeEntities(raw));
	}

	// Pass 2: structure that needs neighbours — setext headings, loose lists.
	const out: string[] = [];
	fence = null;
	for (let i = 0; i < lines.length; i++) {
		const line = lines[i] ?? "";
		if (/^[ \t]*```/.test(line)) {
			fence = fence ? null : "```";
			out.push(line);
			continue;
		}
		if (fence) { out.push(line); continue; }

		const next = lines[i + 1];
		const setext = next === undefined ? null : SETEXT_RE.exec(next);
		if (setext && !isBlank(line) && !LIST_LINE_RE.test(line) && !/^[ \t]*(#|>|\||:::)/.test(line) && line !== "---") {
			const level = (setext[1] ?? "=").startsWith("=") ? 1 : 2;
			out.push(`${"#".repeat(level)} ${line.trim()}`);
			i++;
			continue;
		}

		if (isBlank(line)) {
			// Drop blank lines between two items of one list ("loose" lists).
			let j = i;
			while (j < lines.length && isBlank(lines[j])) j++;
			const prev = out[out.length - 1];
			const following = lines[j];
			if (prev !== undefined && following !== undefined && LIST_LINE_RE.test(prev) && LIST_LINE_RE.test(following)) {
				i = j - 1;
				continue;
			}
		}
		out.push(line);
	}
	return out.join("\n");
};

const MARKDOWN_SIGNALS: RegExp[] = [
	/^[ \t]*#{1,6}[ \t]+\S/m,
	/^[ \t]*(?:[-*+]|\d+[.)])[ \t]+\S/m,
	/^[ \t]*>[ \t]?\S/m,
	/^[ \t]*(?:```|~~~)/m,
	/^[ \t]*\|?[ \t]*:?-+:?[ \t]*\|[ \t:|-]*$/m,
	/\[[^\]\n]+\]\([^)\n]+\)/,
	/(?:https?:\/\/|www\.)[^\s<>]+/i,
	/\*\*[^*\n]+\*\*/,
	/~~[^~\n]+~~/,
];

// True when pasted plain text carries Markdown worth converting rather than inserting
// verbatim. Deliberately conservative: prose with a stray "*" stays as typed.
export const looksLikeMarkdown = (text: string): boolean => MARKDOWN_SIGNALS.some((re) => re.test(text));
