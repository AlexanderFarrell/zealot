import type { Item } from "../../api/item";
import BaseElement from "../common/base_element";

// import "prosemirror-view/style/prosemirror.css";
// import "prosemirror-menu/style/menu.css";
// import "prosemirror-example-setup/style/style.css";

// import {Schema, DOMParser} from "prosemirror-model";
// import {schema} from "prosemirror-schema-basic";
// import {addListNodes} from "prosemirror-schema-list";
import { EditorState } from "prosemirror-state";
import { EditorView } from "prosemirror-view";
import {exampleSetup} from "prosemirror-example-setup";

import {schema, defaultMarkdownParser, defaultMarkdownSerializer} from "prosemirror-markdown";
import API from "../../api/api";
import Popups from "../../core/popups";


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
				plugins: exampleSetup({ schema })
			}),
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