import type { Item } from "../../../api/item";
import BaseElement from "../../../shared/base_element";
import API from "../../../api/api";
import Popups from "../../../shared/popups";
import createZealotEditorView from "../../zealotscript/zealotscript_editor";

class ContentView extends BaseElement<Item> {
	focusEditor() {
		(this.querySelector('[name="editor_prose"]')! as HTMLDivElement).focus();
		console.log("Focused editor");
	}

	render() {
		this.innerHTML = `
			<div name="editor"></div>
		`
		const editorContainer = this.querySelector('[name="editor"]')! as HTMLDivElement;

		createZealotEditorView(editorContainer, {
			content: this.data!.content,
			handleTab: true,
			debounceMs: 1000,
			onUpdate: async (nextContent) => {
				if (nextContent !== this.data!.content) {
					this.data!.content = nextContent;
					await API.item.update(this.data!.item_id, {content: nextContent});
					Popups.add("Saved content", "note", 2);
				}
			}
		})
	}
}

customElements.define("content-view", ContentView);

export default ContentView;