import type { Item } from "../api/item";
import AddItemScoped from "./add_item_scope";
import BaseElement from "./common/base_element";
import ItemView from "./item_view";

class ItemListView extends BaseElement<Item[]> {
	private add_item_attrs: any | null = null;
	private on_add: Function | null = null;

	render() {
		let items = this.data!;

		this.innerHTML = "";
		items.forEach(item => {
			let view = new ItemView().init(item);
			this.appendChild(view)
		})

		if (items.length == 0) {
			this.innerText = "No items";
		}


		if (this.add_item_attrs !== null) {
			let add_view = new AddItemScoped().init(this.add_item_attrs) as AddItemScoped;
			this.prepend(add_view);
			add_view.listen_on_submit(() => {
				if (this.on_add) {
					this.on_add();
				}
				this.render();
			})
		}
	}

	enable_add_item(attributes: any, on_add: Function) {
		this.add_item_attrs = attributes;
		if (this.is_rendered) {
			this.render();
		}
		this.on_add = on_add;
		return this;
	}
}

customElements.define('item-list-view', ItemListView);

export default ItemListView;