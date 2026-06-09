import { Schema, type MarkSpec, type NodeSpec } from "prosemirror-model";
import { schema as baseSchema } from "prosemirror-schema-basic";
import { addListNodes } from "prosemirror-schema-list";

const admonitionSpec: NodeSpec = {
	group: "block",
	content: "block+",
	attrs: {
		kind: { default: "note" }
	},
	defining: true,
	parseDOM: [
		{
			tag: "div[data-admonition]",
			getAttrs: (e) => {
				const element = e as HTMLElement;
				return { kind: element.getAttribute("data-admonition") ?? "note" };
			}
		}
	],
	toDOM(node) {
		const kind = node.attrs.kind || "note";
		return ["div", { "data-admonition": kind, class: `admonition admonition-${kind}` }, 0];
	}
};

const tableSpec: NodeSpec = {
	group: "block",
	content: "table_row+",
	isolating: true,
	parseDOM: [{ tag: "table" }],
	toDOM() {
		return ["table", ["tbody", 0]];
	}
};

const tableRowSpec: NodeSpec = {
	content: "(table_cell | table_header)+",
	parseDOM: [{ tag: "tr" }],
	toDOM() {
		return ["tr", 0];
	}
};

const tableCellSpec: NodeSpec = {
	content: "block+",
	attrs: { colspan: { default: 1 }, rowspan: { default: 1 } },
	isolating: true,
	parseDOM: [
		{
			tag: "td",
			getAttrs: (e) => {
				const element = e as HTMLElement;
				return {
					colspan: Number(element.getAttribute("colspan") || 1),
					rowspan: Number(element.getAttribute("rowspan") || 1),
				};
			}
		}
	],
	toDOM(node) {
		return ["td", { colspan: node.attrs.colspan, rowspan: node.attrs.rowspan }, 0];
	}
};

const tableHeaderSpec: NodeSpec = {
	content: "block+",
	attrs: { colspan: { default: 1 }, rowspan: { default: 1 } },
	isolating: true,
	parseDOM: [
		{
			tag: "th",
			getAttrs: (e) => {
				const element = e as HTMLElement;
				return {
					colspan: Number(element.getAttribute("colspan") || 1),
					rowspan: Number(element.getAttribute("rowspan") || 1),
				};
			}
		}
	],
	toDOM(node) {
		return ["th", { colspan: node.attrs.colspan, rowspan: node.attrs.rowspan }, 0];
	}
};

const getCodeBlockLanguage = (element: HTMLElement): string => {
	const direct = element.getAttribute("data-language") || "";
	if (direct.trim().length > 0) return direct.trim();
	const className = element.getAttribute("class") || "";
	const classMatch = /(?:^|\s)language-([A-Za-z0-9_+-]+)(?:\s|$)/.exec(className);
	if (classMatch?.[1]) return classMatch[1];
	const codeEl = element.querySelector("code");
	if (codeEl) {
		const codeClass = codeEl.getAttribute("class") || "";
		const codeMatch = /(?:^|\s)language-([A-Za-z0-9_+-]+)(?:\s|$)/.exec(codeClass);
		if (codeMatch?.[1]) return codeMatch[1];
	}
	return "";
};

const codeBlockSpec: NodeSpec = {
	...(baseSchema.spec.nodes.get("code_block") as NodeSpec),
	attrs: { language: { default: "" } },
	parseDOM: [
		{
			tag: "pre",
			preserveWhitespace: "full" as const,
			getAttrs: (e) => {
				const element = e as HTMLElement;
				return { language: getCodeBlockLanguage(element) };
			}
		}
	],
	toDOM(node) {
		const language = (node.attrs.language || "").trim();
		const codeAttrs = language.length > 0 ? { class: `language-${language}` } : {};
		const preAttrs = language.length > 0 ? { "data-language": language } : {};
		return ["pre", preAttrs, ["code", codeAttrs, 0]];
	}
};

const strikeMark: MarkSpec = {
	parseDOM: [{ tag: "s" }, { tag: "del" }],
	toDOM() { return ["s", 0]; }
};

const underlineMark: MarkSpec = {
	parseDOM: [{ tag: "u" }],
	toDOM() { return ["u", 0]; }
};

const highlightMark: MarkSpec = {
	parseDOM: [{ tag: "mark" }],
	toDOM() { return ["mark", 0]; }
};

const subscriptMark: MarkSpec = {
	parseDOM: [{ tag: "sub" }],
	toDOM() { return ["sub", 0]; }
};

const superscriptMark: MarkSpec = {
	parseDOM: [{ tag: "sup" }],
	toDOM() { return ["sup", 0]; }
};

const SAFE_COLOR_RE = /^(?:[a-zA-Z]+|#(?:[0-9a-fA-F]{3}|[0-9a-fA-F]{6})|rgb\(\s*\d+\s*,\s*\d+\s*,\s*\d+\s*\)|hsl\(\s*\d+\s*,\s*\d+%\s*,\s*\d+%\s*\))$/;

const colorMark: MarkSpec = {
	attrs: { value: { default: "" } },
	parseDOM: [
		{
			tag: "span.zealot-color[style]",
			getAttrs: (e) => {
				const el = e as HTMLElement;
				const color = el.style.color || "";
				return SAFE_COLOR_RE.test(color.trim()) ? { value: color.trim() } : false;
			}
		}
	],
	toDOM(mark) {
		const value = (mark.attrs.value || "").trim();
		const safe = SAFE_COLOR_RE.test(value) ? value : "";
		return ["span", { style: safe ? `color: ${safe}` : "", class: "zealot-color" }, 0];
	}
};

const youtubeEmbedSpec: NodeSpec = {
	group: "block",
	atom: true,
	attrs: { videoId: { default: "" } },
	parseDOM: [
		{
			tag: "div[data-youtube]",
			getAttrs: (e) => ({
				videoId: (e as HTMLElement).getAttribute("data-youtube") ?? "",
			})
		}
	],
	toDOM(node) {
		return ["div", { "data-youtube": node.attrs.videoId, class: "zealot-youtube-embed" }];
	}
};

const mathInlineSpec: NodeSpec = {
	group: "inline",
	inline: true,
	atom: true,
	attrs: { src: { default: "" } },
	parseDOM: [
		{
			tag: "span[data-math-inline]",
			getAttrs: (e) => ({ src: (e as HTMLElement).getAttribute("data-math-inline") ?? "" })
		}
	],
	toDOM(node) {
		return ["span", { "data-math-inline": node.attrs.src, class: "zealot-math-inline" }];
	}
};

const mathBlockSpec: NodeSpec = {
	group: "block",
	atom: true,
	attrs: { src: { default: "" } },
	parseDOM: [
		{
			tag: "div[data-math-block]",
			getAttrs: (e) => ({ src: (e as HTMLElement).getAttribute("data-math-block") ?? "" })
		}
	],
	toDOM(node) {
		return ["div", { "data-math-block": node.attrs.src, class: "zealot-math-block" }];
	}
};

const iconRefSpec: NodeSpec = {
	group: "inline",
	inline: true,
	atom: true,
	attrs: { name: { default: "" } },
	parseDOM: [
		{
			tag: "span[data-icon-ref]",
			getAttrs: (e) => ({ name: (e as HTMLElement).getAttribute("data-icon-ref") ?? "" })
		}
	],
	toDOM(node) {
		const name = node.attrs.name as string;
		return ["span", { "data-icon-ref": name, class: "zealot-icon", title: name }, `:${name}:`];
	}
};

const imageSpec: NodeSpec = {
	group: "inline",
	inline: true,
	atom: true,
	attrs: {
		src: { default: "" },
		alt: { default: "" },
		title: { default: "" },
	},
	parseDOM: [{
		tag: "img[src]",
		getAttrs: (e) => {
			const el = e as HTMLImageElement;
			return {
				src: el.getAttribute("src") ?? "",
				alt: el.getAttribute("alt") ?? "",
				title: el.getAttribute("title") ?? "",
			};
		}
	}],
	toDOM(node) {
		const attrs: Record<string, string> = { src: node.attrs.src as string, class: "zealot-image" };
		if (node.attrs.alt) attrs.alt = node.attrs.alt as string;
		if (node.attrs.title) attrs.title = node.attrs.title as string;
		return ["img", attrs];
	}
};

const dateRefSpec: NodeSpec = {
	group: "inline",
	inline: true,
	atom: true,
	attrs: {
		raw: { default: "" },
		kind: { default: "day" },
	},
	parseDOM: [
		{
			tag: "span[data-date-ref]",
			getAttrs: (e) => {
				const el = e as HTMLElement;
				const raw = el.getAttribute("data-date-ref") ?? "";
				const kind = el.getAttribute("data-date-kind") ?? "day";
				return { raw, kind };
			}
		}
	],
	toDOM(node) {
		const raw = node.attrs.raw as string;
		const kind = node.attrs.kind as string;
		return ["span", { "data-date-ref": raw, "data-date-kind": kind, class: `zealot-date-ref zealot-date-ref-${kind}` }, raw];
	}
};

const listItemSpec: NodeSpec = {
	content: "paragraph block*",
	attrs: { checked: { default: null } },
	defining: true,
	parseDOM: [
		{
			tag: "li",
			getAttrs: (e) => {
				const el = e as HTMLElement;
				const raw = el.getAttribute("data-checked");
				if (raw === null) return { checked: null };
				return { checked: raw === "true" };
			}
		}
	],
	toDOM(node) {
		const checked: boolean | null = node.attrs.checked;
		if (checked === null) return ["li", 0];
		const inputAttrs: Record<string, string> = {
			type: "checkbox",
			class: "zealot-task-checkbox",
		};
		if (checked) inputAttrs["checked"] = "";
		return ["li", { "data-checked": String(checked), class: "zealot-task-item" }, ["input", inputAttrs], 0];
	}
};

const detailsSpec: NodeSpec = {
	group: "block",
	content: "block+",
	attrs: { summary: { default: "" } },
	defining: true,
	parseDOM: [{
		tag: "details.zealot-details",
		getAttrs: (e) => ({
			summary: (e as HTMLElement).querySelector("summary")?.textContent?.trim() ?? ""
		})
	}],
	toDOM(node) {
		return ["details", { class: "zealot-details" },
			["summary", {}, node.attrs.summary || ""],
			["div", { class: "zealot-details-content" }, 0]];
	}
};

const spoilerSpec: NodeSpec = {
	group: "block",
	content: "block+",
	defining: true,
	parseDOM: [{ tag: "details.zealot-spoiler" }],
	toDOM() {
		return ["details", { class: "zealot-spoiler" },
			["summary", {}, "Show spoiler"],
			["div", { class: "zealot-spoiler-content" }, 0]];
	}
};

const definitionListSpec: NodeSpec = {
	group: "block",
	content: "(definition_term definition_desc+)+",
	parseDOM: [{ tag: "dl.zealot-definition" }],
	toDOM() { return ["dl", { class: "zealot-definition" }, 0]; }
};

const definitionTermSpec: NodeSpec = {
	content: "inline*",
	parseDOM: [{ tag: "dl.zealot-definition > dt" }],
	toDOM() { return ["dt", 0]; }
};

const definitionDescSpec: NodeSpec = {
	content: "block+",
	parseDOM: [{ tag: "dl.zealot-definition > dd" }],
	toDOM() { return ["dd", 0]; }
};

const columnsSpec: NodeSpec = {
	group: "block",
	content: "column{2,4}",
	defining: true,
	parseDOM: [{ tag: "div.zealot-columns" }],
	toDOM(node) {
		const n = Math.max(node.childCount, 2);
		return ["div", { class: `zealot-columns zealot-columns-${n}` }, 0];
	}
};

const columnSpec: NodeSpec = {
	content: "block+",
	defining: true,
	parseDOM: [{ tag: "div.zealot-column" }],
	toDOM() { return ["div", { class: "zealot-column" }, 0]; }
};

const tabsSpec: NodeSpec = {
	group: "block",
	content: "tab+",
	defining: true,
	parseDOM: [{ tag: "div.zealot-tabs" }],
	toDOM() { return ["div", { class: "zealot-tabs" }, 0]; }
};

const tabSpec: NodeSpec = {
	content: "block+",
	attrs: { title: { default: "" } },
	defining: true,
	parseDOM: [{
		tag: "div.zealot-tab-panel",
		getAttrs: (e) => ({ title: (e as HTMLElement).getAttribute("data-tab-title") ?? "" })
	}],
	toDOM(node) {
		return ["div", { class: "zealot-tab-panel", "data-tab-title": node.attrs.title as string }, 0];
	}
};

const nodes = addListNodes(
	baseSchema.spec.nodes.update("code_block", codeBlockSpec).update("list_item", listItemSpec),
	"paragraph block*",
	"block"
).append({
	admonition: admonitionSpec,
	table: tableSpec,
	table_row: tableRowSpec,
	table_cell: tableCellSpec,
	table_header: tableHeaderSpec,
	youtube_embed: youtubeEmbedSpec,
	math_inline: mathInlineSpec,
	math_block: mathBlockSpec,
	icon_ref: iconRefSpec,
	image: imageSpec,
	date_ref: dateRefSpec,
	details: detailsSpec,
	spoiler: spoilerSpec,
	definition_list: definitionListSpec,
	definition_term: definitionTermSpec,
	definition_desc: definitionDescSpec,
	columns: columnsSpec,
	column: columnSpec,
	tabs: tabsSpec,
	tab: tabSpec,
});

const marks = baseSchema.spec.marks.append({
	strike: strikeMark,
	underline: underlineMark,
	highlight: highlightMark,
	subscript: subscriptMark,
	superscript: superscriptMark,
	color: colorMark,
});

export const ZealotSchema = new Schema({ nodes, marks });
export default ZealotSchema;
