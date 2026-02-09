import { Schema, type MarkSpec, type NodeSpec } from "prosemirror-model";
import {schema as baseSchema} from "prosemirror-schema-basic";
import {addListNodes} from "prosemirror-schema-list";

const admonitionSpec: NodeSpec = {
	group: "block",
	content: "block+",
	attrs: {
		kind: {default: "note"}
	},
	defining: true,
	parseDOM: [
		{
			tag: "div[data-admonition]",
			getAttrs: (e) => {
				const element = e as HTMLElement;
				return {
					kind: element.getAttribute("data-admonition") ?? "note"
				}
			}
		}
	],
	toDOM(node) {
		const kind = node.attrs.kind || "note";
		return [
			"div",
			{
				"data-admonition": kind,
				class: `admonition admonition=${kind}`
			},
			0
		]
	}
}

const tableSpec: NodeSpec = {
	group: "block",
	content: "table_row+",
	isolating: true,
	tableRole: "table",
	parseDOM: [{tag: "table"}],
	toDOM() {
		return ["table", ["tbody", 0]];
	}
}

const tableRowSpec: NodeSpec = {
	content: "(table_cell | table_header)",
	tableRole: "row",
	parseDOM: [{tag: "tr"}],
	toDOM() {
		return ["tr", 0]
	}
}

const tableCellSpec: NodeSpec = {
	content: "block+",
	attrs: {
		colspan: {default: 1},
		rowspan: {default: 1}
	},
	isolating: true,
	tableRole: "cell",
	parseDOM: [
		{
			tag: "td",
			getAttrs: (e) => {
				const element = e as HTMLElement;
				return {
					colspan: Number(element.getAttribute('colspan') || 1),
					rowspan: Number(element.getAttribute('rowspan') || 1),
				}
			}
		}
	],
	toDOM(node) {
		return ["td", {colspan: node.attrs.colspan, rowspan: node.attrs.rowspan}, 0]
	}
}

const tableHeaderSpec: NodeSpec = {
	content: "block+",
	attrs: {
		colspan: {default: 1},
		rowspan: {default: 1}
	},
	isolating: true,
	tableRole: "header_cell",
	parseDOM: [
		{
			tag: "th",
			getAttrs: (e) => {
				const element = e as HTMLElement;
				return {
					colspan: Number(element.getAttribute('colspan') || 1),
					rowspan: Number(element.getAttribute("rowspan") || 1)
				}
			}
		}
	],
	toDOM(node) {
		return ["th", {colspan: node.attrs.colspan, rowspan: node.attrs.rowspan}, 0]
	}
}

const itemLinkSpec: NodeSpec = {
	inline: true,
	group: "inline",
	atom: true,
	attrs: {
		title: {default: ""},
		href: {default: ""}
	},
	parseDOM: [
		{
			tag: "a[data-itemlink]",
			getAttrs: (e) => {
				const element = e as HTMLElement;
				return {
					title: element.textContent || "",
					href: element.getAttribute("href")
				}
			}
		}
	],
	toDOM(node) {
		return [
			"a",
			{
				href: node.attrs.href,
				"data-itemlink": true,
				class: "itemlink"
			},
			node.attrs.title
		]
	}
}

const strikeMark: MarkSpec = {
	parseDOM: [{tag: "s"}, {tag: "del"}],
	toDOM() {
		return ["s", 0];
	}
}

const underlineMark: MarkSpec = {
	parseDOM: [{tag: "u"}],
	toDOM() {
		return ["u", 0]
	}
}

const highlightMark: MarkSpec = {
	parseDOM: [{tag: "mark"}],
	toDOM() {
		return ["sub", 0]
	}
}

const subscriptMark: MarkSpec = {
	parseDOM: [{tag: "sub"}],
	toDOM() {
		return ["sub", 0]
	}
}

const superscriptMark: MarkSpec = {
	parseDOM: [{tag: "sup"}],
	toDOM() {
		return ["sup", 0];
	}
}


const nodes = addListNodes(baseSchema.spec.nodes, "paragraph block*", "block").append({
	admonition: admonitionSpec,
	table: tableSpec,
	table_row: tableRowSpec,
	table_cell: tableCellSpec,
	table_header: tableHeaderSpec,
	itemlink: itemLinkSpec
})

const marks = baseSchema.spec.marks.append({
	strike: strikeMark,
	underline: underlineMark,
	highlight: highlightMark,
	subscript: subscriptMark,
	superscript: superscriptMark,
})

export const ZealotSchema = new Schema({nodes, marks});
export default ZealotSchema;