// Bare-URL detection shared by the parser (stored Markdown), paste import, and typing.

const URL_START_RE = /^(?:https?:\/\/|www\.)[^\s<>]/i;
const URL_BODY_RE = /^(?:https?:\/\/|www\.)[^\s<>]+/i;
const TRAILING_PUNCTUATION = ".,;:!?'\"*_~";

const countChar = (text: string, char: string): number => {
	let count = 0;
	for (const c of text) if (c === char) count++;
	return count;
};

// A URL may start after whitespace, an opening bracket, or punctuation such as
// "docs:https://x" (a link jammed against a label at the end of a bullet).
export const isUrlStart = (text: string, index: number): boolean => {
	if (!URL_START_RE.test(text.slice(index, index + 12))) return false;
	if (index === 0) return true;
	const before = text[index - 1] ?? "";
	return !/[A-Za-z0-9/@.#=&?%+_-]/.test(before);
};

// Returns the URL at the start of `text`, excluding sentence punctuation and an
// unbalanced closing bracket, so "(see https://a.b/c)." links only https://a.b/c.
export const matchBareUrl = (text: string): string | null => {
	const m = URL_BODY_RE.exec(text);
	if (!m) return null;
	let url = m[0];
	for (;;) {
		const last = url[url.length - 1] ?? "";
		if (last.length > 0 && TRAILING_PUNCTUATION.includes(last)) { url = url.slice(0, -1); continue; }
		if (last === ")" && countChar(url, ")") > countChar(url, "(")) { url = url.slice(0, -1); continue; }
		if (last === "]" && countChar(url, "]") > countChar(url, "[")) { url = url.slice(0, -1); continue; }
		break;
	}
	// Require something after the scheme/host prefix.
	if (/^(?:https?:\/\/|www\.)$/i.test(url)) return null;
	return url;
};

// Link destinations for bare "www." URLs need a scheme to be clickable.
export const hrefForUrl = (url: string): string => (/^www\./i.test(url) ? `https://${url}` : url);
