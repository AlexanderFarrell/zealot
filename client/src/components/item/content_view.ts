import type { Item } from "../../api/item";
import BaseElement from "../common/base_element";


import {Schema, DOMParser} from "prosemirror-model";
// import {schema} from "prosemirror-schema-basic";
import {addListNodes} from "prosemirror-schema-list";
import { EditorState } from "prosemirror-state";
import { EditorView } from "prosemirror-view";
import {exampleSetup} from "prosemirror-example-setup";

import {schema, defaultMarkdownParser, defaultMarkdownSerializer} from "prosemirror-markdown";


class ContentView extends BaseElement<Item> {
	render() {
		this.innerHTML = `
		<div name="editor"></div>
		<!--<textarea name="content"></textarea>-->l
		`


		let editor_container = this.querySelector('[name="editor"]')!;
		let content_container = this.querySelector('[name="content"]')!;
		
		let view = new EditorView(editor_container, {
			state: EditorState.create({
				doc: defaultMarkdownParser.parse(this.data!.content),
				plugins: exampleSetup({schema: schema})
			})
		})

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