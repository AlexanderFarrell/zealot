import type { Item } from "../../api/item";
import BaseElement from "../common/base_element";

// import "prosemirror-view/style/prosemirror.css";
// import "prosemirror-menu/style/menu.css";
// import "prosemirror-example-setup/style/style.css";

// import {Schema, DOMParser} from "prosemirror-model";
// import {schema} from "prosemirror-schema-basic";
// import {addListNodes} from "prosemirror-schema-list";
import { EditorState, Plugin } from "prosemirror-state";
import { Decoration, DecorationSet, EditorView } from "prosemirror-view";
import {exampleSetup} from "prosemirror-example-setup";

import {schema, defaultMarkdownParser, defaultMarkdownSerializer} from "prosemirror-markdown";
import hljs from "highlight.js/lib/common";
import "highlight.js/styles/github-dark.css";
import API from "../../api/api";
import Popups from "../../core/popups";

function buildInlineHighlightDecorations(html: string, basePos: number): Decoration[] {
	const container = document.createElement("div");
	container.innerHTML = html;

	const decorations: Decoration[] = [];
	let offset = 0;

	const walk = (node: ChildNode, classStack: string[]) => {
		if (node.nodeType === Node.TEXT_NODE) {
			const text = node.textContent ?? "";
			if (text.length > 0 && classStack.length > 0) {
				decorations.push(
					Decoration.inline(
						basePos + offset,
						basePos + offset + text.length,
						{ class: classStack.join(" ") }
					)
				);
			}
			offset += text.length;
			return;
		}

		if (node.nodeType !== Node.ELEMENT_NODE) return;

		const element = node as HTMLElement;
		const nextStack = classStack.concat(Array.from(element.classList));
		for (const child of Array.from(element.childNodes)) {
			walk(child, nextStack);
		}
	};

	for (const child of Array.from(container.childNodes)) {
		walk(child, []);
	}

	return decorations;
}

function buildCodeBlockDecorations(doc: EditorState["doc"]): DecorationSet {
	const decorations: Decoration[] = [];

	doc.descendants((node, pos) => {
		if (node.type.name !== "code_block") return;

		decorations.push(
			Decoration.node(pos, pos + node.nodeSize, { class: "hljs" })
		);

		const text = node.textContent;
		if (text.length === 0) return;

		const result = hljs.highlightAuto(text);
		const inlineDecorations = buildInlineHighlightDecorations(result.value, pos + 1);
		decorations.push(...inlineDecorations);
	});

	return DecorationSet.create(doc, decorations);
}

const codeHighlightPlugin: any = new Plugin({
	state: {
		init: (_, state) => buildCodeBlockDecorations(state.doc),
		apply: (tr, value, _oldState, newState) => {
			if (!tr.docChanged) return value;
			return buildCodeBlockDecorations(newState.doc);
		}
	},
	props: {
		decorations(state) {
			return codeHighlightPlugin.getState(state);
		}
	}
});


class ContentView extends BaseElement<Item> {
	private save_timer: number | null = null;

	focusEditor() {
		(this.querySelector('[name="editor_prose"]')! as HTMLDivElement).focus();
		console.log("Focused editor");
	}

	render() {
		this.innerHTML = `
		<div name="editor"></div>
		<!--<textarea name="content"></textarea>-->
		`


		let editor_container = this.querySelector('[name="editor"]')!;

		const view = new EditorView(editor_container, {
			state: EditorState.create({
				doc: defaultMarkdownParser.parse(this.data!.content),
				plugins: exampleSetup({ schema })/*.concat(codeHighlightPlugin)*/
			}),
			handleKeyDown: (view, event) => {
				if (event.key !== "Tab") return false;
				event.preventDefault();
				view.dispatch(view.state.tr.insertText("\t"));
				return true;
			},
			dispatchTransaction: (tr) => {
				const next_state = view.state.apply(tr);
				view.updateState(next_state);

				if (this.save_timer !== null) clearTimeout(this.save_timer);
				this.save_timer = setTimeout(async () => {
					const next_markdown = defaultMarkdownSerializer.serialize(view.state.doc);
					if (next_markdown !== this.data!.content) {
						this.data!.content = next_markdown;
						await API.item.update(this.data!.item_id, {content: next_markdown});
						Popups.add("Saved content", 'note', 2)
					}
				}, 1000)
			}
		})
		view.dom.setAttribute('name', 'editor_prose')



		// let content_container = this.querySelector('[name="content"]')!;
		
		// let view = new EditorView(editor_container, {
		// 	state: EditorState.create({
		// 		doc: defaultMarkdownParser.parse(this.data!.content),
		// 		plugins: exampleSetup({schema: schema})
		// 	})
		// })

		// content_container.innerHTML = this.data!.content;

		// const mySchema = new Schema({
		// 	nodes: addListNodes(schema.spec.nodes, "paragraph block*", "block"),
		// 	marks: schema.spec.marks
		// });

		// let view = new EditorView(editor_container, {
		// 	state: EditorState.create({
		// 		doc: DOMParser.fromSchema(mySchema).parse(content_container),
		// 		plugins: exampleSetup({schema: mySchema})
		// 	})
		// })
	}
}

customElements.define('content-view', ContentView)

export default ContentView;
