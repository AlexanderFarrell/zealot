// Shared syntax highlighter for ZealotScript code blocks.
//
// highlight.js is lazy-loaded (like katex/mermaid elsewhere) so it stays out of
// the initial bundle. Two consumers:
//   - the read-only <zealotscript-view>, which swaps <code> innerHTML for the
//     highlighted markup (`highlightToHtml`);
//   - the live editor, which cannot touch the contenteditable DOM directly and
//     instead maps highlight.js output into flat token ranges for ProseMirror
//     inline decorations (`highlightToTokens`).
//
// Both paths share the same `hljs-*` class names, styled in tags.scss.

import type { HLJSApi } from "highlight.js";

let hljsLib: HLJSApi | null = null;
let hljsLoading: Promise<HLJSApi> | null = null;

// Curated language set — covers the common cases without pulling the full
// ~200-grammar "highlight.js" barrel into the bundle. Keys include the aliases
// we expect from fence tags.
const LANGUAGE_LOADERS: Record<string, () => Promise<{ default: unknown }>> = {
	javascript: () => import("highlight.js/lib/languages/javascript"),
	typescript: () => import("highlight.js/lib/languages/typescript"),
	json: () => import("highlight.js/lib/languages/json"),
	bash: () => import("highlight.js/lib/languages/bash"),
	shell: () => import("highlight.js/lib/languages/shell"),
	python: () => import("highlight.js/lib/languages/python"),
	rust: () => import("highlight.js/lib/languages/rust"),
	xml: () => import("highlight.js/lib/languages/xml"),
	css: () => import("highlight.js/lib/languages/css"),
	scss: () => import("highlight.js/lib/languages/scss"),
	sql: () => import("highlight.js/lib/languages/sql"),
	yaml: () => import("highlight.js/lib/languages/yaml"),
	markdown: () => import("highlight.js/lib/languages/markdown"),
};

// Aliases → canonical language key registered above.
const LANGUAGE_ALIASES: Record<string, string> = {
	js: "javascript",
	jsx: "javascript",
	mjs: "javascript",
	cjs: "javascript",
	ts: "typescript",
	tsx: "typescript",
	sh: "bash",
	zsh: "bash",
	shellscript: "shell",
	py: "python",
	rs: "rust",
	html: "xml",
	svg: "xml",
	yml: "yaml",
	md: "markdown",
};

const canonicalLanguage = (lang: string): string => {
	const key = lang.trim().toLowerCase();
	return LANGUAGE_ALIASES[key] ?? key;
};

const loadHighlighter = async (): Promise<HLJSApi> => {
	if (hljsLib) return hljsLib;
	if (hljsLoading) return hljsLoading;
	hljsLoading = (async () => {
		const core = (await import("highlight.js/lib/core")).default;
		await Promise.all(
			Object.entries(LANGUAGE_LOADERS).map(async ([name, load]) => {
				const mod = await load();
				// registerLanguage is idempotent for our purposes.
				core.registerLanguage(name, mod.default as never);
			}),
		);
		hljsLib = core;
		return core;
	})();
	return hljsLoading;
};

// Ensure the library is ready. Returns null if it is still loading (so callers
// can render plain text now and re-render once `whenReady` resolves).
export const highlighterOrNull = (): HLJSApi | null => hljsLib;

export const whenHighlighterReady = (): Promise<HLJSApi> => loadHighlighter();

const isSupported = (lang: string): boolean => {
	const canonical = canonicalLanguage(lang);
	return canonical.length > 0 && Object.prototype.hasOwnProperty.call(LANGUAGE_LOADERS, canonical);
};

// --- Read-only view: produce highlighted HTML --------------------------------

// Highlights `code` and returns a safe HTML string of hljs-classed spans.
// Returns null when the library is not yet loaded or the language is unknown
// (callers fall back to the plain, already-escaped text content).
export const highlightToHtml = (code: string, lang: string): string | null => {
	const lib = hljsLib;
	if (!lib) return null;
	const canonical = canonicalLanguage(lang);
	try {
		if (isSupported(canonical)) {
			return lib.highlight(code, { language: canonical, ignoreIllegals: true }).value;
		}
		if (lang.trim().length === 0) {
			// No fence language: let highlight.js auto-detect among registered langs.
			return lib.highlightAuto(code).value;
		}
	} catch {
		return null;
	}
	return null;
};

// --- Editor: produce flat token ranges for ProseMirror decorations -----------

export interface HighlightToken {
	from: number; // char offset within the code block's text
	to: number;
	className: string; // space-joined hljs-* classes
}

// Highlights `code` and flattens the resulting hljs span tree into non-nested
// ranges. Offsets are plain UTF-16 string indices into `code`, which line up
// with ProseMirror text positions inside a code block (a single text node).
export const highlightToTokens = (code: string, lang: string): HighlightToken[] | null => {
	const html = highlightToHtml(code, lang);
	if (html === null) return null;

	const template = document.createElement("template");
	template.innerHTML = html;

	const tokens: HighlightToken[] = [];
	let offset = 0;

	const walk = (node: Node, inherited: string): void => {
		for (const child of Array.from(node.childNodes)) {
			if (child.nodeType === Node.TEXT_NODE) {
				const text = child.textContent ?? "";
				if (text.length > 0) {
					if (inherited.length > 0) {
						tokens.push({ from: offset, to: offset + text.length, className: inherited });
					}
					offset += text.length;
				}
			} else if (child.nodeType === Node.ELEMENT_NODE) {
				const el = child as HTMLElement;
				// hljs nests spans; concatenate ancestor classes so the innermost
				// (most specific) token still carries its parent scope classes.
				const cls = el.getAttribute("class") ?? "";
				const combined = [inherited, cls].filter((c) => c.length > 0).join(" ");
				walk(el, combined);
			}
		}
	};

	walk(template.content, "");
	return tokens;
};
