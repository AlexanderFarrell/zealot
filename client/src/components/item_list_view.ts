import type { Item } from "../api/item";
import { FilterIcon, GroupByIcon } from "../assets/asset_map";
import AddItemScoped from "./add_item_scope";
import BaseElement from "./common/base_element";
import type ButtonGroup from "./common/button_group";
import { ButtonDef } from "./common/button_group";
import type ItemAnalysis from "./item_analysis";
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
		<item-analysis></item-analysis>
		<div name="list_view"></div>
		`;
		let add_item_view = this.querySelector('add-item-scoped')! as AddItemScoped;
		let analysis = this.querySelector('item-analysis')! as ItemAnalysis;
		analysis.init(items);
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

		let view_containers: Map<string, Item[]> = new Map();
		let uncategorized: Item[] = []


		// Setup list view

		items.forEach(item => {
			if (item.attributes!['Status'] != null) {
				if (!view_containers.has(item.attributes!['Status'])) {
					view_containers.set(item.attributes!['Status'], [])
				}
				view_containers.get(item.attributes!['Status'])?.push(item)
			} else {
				uncategorized.push(item)
			}
		})

		let render_category = (status: string, isOpen: boolean = true) => {
			if (view_containers.get(status) == null) {
				return;
			}
			let div = document.createElement('div');
			div.innerHTML = `
			<h2>${status}</h2>
			<div name="items"></div>
			`
			let i_container = div.querySelector('[name="items"]')! as HTMLDivElement;
			let heading = div.querySelector('h2')! as HTMLHeadingElement;

			let refresh_container = () => {
				if (isOpen) {
					i_container.innerHTML = "";
					view_containers.get(status)?.forEach(item => {
						i_container.appendChild(new ItemView().init(item));
					})
				} else {
					i_container.innerText = view_containers.get(status)!.length.toString() + ' hidden';
				}
			}
			refresh_container();

			heading.addEventListener('click', () => {
				isOpen = !isOpen;
				refresh_container()
			})
			list_view.appendChild(div)
		}

		// Render uncategorized first
		let div = document.createElement('div')
		uncategorized.forEach(item => {
			div.appendChild(new ItemView().init(item));
		})
		list_view.appendChild(div)

		render_category('Working')
		render_category('To Do')
		render_category('Specify')
		render_category('Blocked')
		render_category('Complete', )
		render_category('Hold')
		render_category('Rejected')

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