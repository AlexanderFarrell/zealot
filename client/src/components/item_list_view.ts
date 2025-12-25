import type { Item } from "../api/item";
import { FilterIcon, GroupByIcon } from "../assets/asset_map";
import AddItemScoped from "./add_item_scope";
import BaseElement from "./common/base_element";
import type ButtonGroup from "./common/button_group";
import { ButtonDef } from "./common/button_group";
import ItemView from "./item_view";

class ItemListView extends BaseElement<Item[]> {
	private add_item_attrs: any | null = null;
	private on_add: Function | null = null;

	render() {
		let items = this.data!;
		items = items.sort((a, b) => {
			let a_priority = (a.attributes) ? a.attributes['Priority'] || 0 : 0;
			let b_priority = (b.attributes) ? b.attributes['Priority'] || 0 : 0;
			return b_priority - a_priority;
		})

		this.innerHTML = `
		<add-item-scoped></add-item-scoped>
		<!--<div style="display: grid; grid-template-columns: auto auto;">
			<div name="filter"><img class="icon" src="${FilterIcon}"> Filter</div>
			<div name="group_by"><img class="icon" src="${GroupByIcon}"> Group By</div>
		</div>-->
		<!--<button-group name="sort_filter_buttons"></button-group>-->
		<div name="list_view"></div>
		`;
		let add_item_view = this.querySelector('add-item-scoped')! as AddItemScoped;
		// let filter_buttons = this.querySelector('button-group')! as ButtonGroup;
		let list_view = this.querySelector('[name="list_view"]')! as HTMLDivElement;

		// Setup add item scoped view		
		if (this.add_item_attrs !== null) {
			add_item_view.style.display = "block";
			add_item_view.init(this.add_item_attrs);
			add_item_view.listen_on_submit(() => {
				if (this.on_add) {
					this.on_add();
				}
				this.render();
			})
		} else {
			add_item_view.style.display = "none";
		}

		// Setup filter buttons
		// filter_buttons.init([
		// 	new ButtonDef(
		// 		FilterIcon,
		// 		"Filter",
		// 		() => {}
		// 	),
		// 	new ButtonDef(
		// 		GroupByIcon,
		// 		"Group By",
		// 		() => {}
		// 	)
		// ])


		// Setup list view

		items.forEach(item => {
			let view = new ItemView().init(item);
			this.appendChild(view)
		})

		if (items.length == 0) {
			list_view.innerHTML = "No items";
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