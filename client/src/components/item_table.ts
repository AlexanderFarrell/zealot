import API from "../api/api";
import type { Item } from "../api/item";
import DragUtil from "../core/drag_helper";
import { router } from "../core/router";
import BaseElement, { BaseAPIElement } from "./common/base_element";
import { AttributeValueView } from "./item/attribute_item_view";

interface ItemTableInfo {
	items: Item[],
	columns: string[]
}

class ItemTable extends BaseElement<ItemTableInfo> {
	public item_type: string | null = null;
	private sort_by: string | null = null;
	private descending: boolean = false;

	render() {
		this.innerHTML = `
		<table name="items"></div>
		`

		let container = this.querySelector('[name="items"]')! as HTMLTableElement;

		let headings = ['Title', ...this.data!.columns];


		// container.innerHTML = 
		// "<thead>" +
		// headings.map(i => {
		// 	return `<td>${i}</td>`
		// }).join("")
		// + "</thead>"

		// Headings
		{
			let header_row = document.createElement('thead')
			headings.forEach(h => {
				let td = document.createElement('td')
				td.innerText = h
				td.addEventListener('click', () => {
					if (this.sort_by == h) {
						if (this.descending) {
							this.sort_by = null;
							this.descending = false;
						} else {
							this.descending = true;
						}
					} else {
						this.sort_by = h;
					}
					this.render();
				})
				header_row.appendChild(td)
			})
			container.appendChild(header_row)
		}

		// Add element
		{
			let add_row: HTMLTableRowElement = document.createElement('tr');

			let td = document.createElement('td');
			let title_input = document.createElement('input')
			title_input.name = "title";
			title_input.type = "text";
			td.appendChild(title_input)
			add_row.appendChild(td)

			let field_inputs: Map<string, AttributeValueView> = new Map();

			this.data!.columns.forEach(attr_key => {
				let td = document.createElement('td');
				let view = new AttributeValueView();
				view.init({
					key: attr_key,
					value: ""
				})
				field_inputs.set(attr_key, view)
				view.addEventListener('keydown', (e: KeyboardEvent) => {
					if (e.key == "Enter") {
						setTimeout(async () => {
							try {
								let attrs: any = {}
								if (title_input.value == "") {
									throw new Error("Please enter a title");
								}
								field_inputs.forEach((field: AttributeValueView, key: string) => {
									if (field.value == "") {
										throw new Error("Please enter a value for " + key)
									}
									attrs[key] = field.value
								})
								// Submit
								let item: Item = {
									item_id: -1,
									title: title_input.value,
									content: "",
									attributes: attrs
								}
								item = await API.item.add(item)
								if (this.item_type != null) {

									await API.item.assign_type(item.item_id, this.item_type)
								}
								this.data!.items.push(item)
								this.render();
								setTimeout(() => {
									(this.querySelector('[name="title"]')! as HTMLInputElement).focus();
								}, 50);
							} catch (e) {
								console.error(e)
							}
						}, 50);
					} 
				})
				td.appendChild(view)
				add_row.appendChild(td)
			})
			container.appendChild(add_row)
		}
		
		let items = [...this.data!.items];
		// Sort if needed
		if (this.sort_by != null) {
			items = items.sort((a, b) => {
				if (this.sort_by == "Title") {
					return a.title.localeCompare(b.title);
				}
				return `${a.attributes![this.sort_by!]}`.localeCompare(`${b.attributes![this.sort_by!]}`);
			})
			if (this.descending) {
				items = items.reverse()
			}
		}

		items.forEach(item => {
			let element: HTMLTableRowElement = document.createElement('tr');

			// First, make the link as the title
			let td_first = document.createElement('td')
			let title_link = document.createElement('a')
			title_link.innerText = item.title;
			td_first.addEventListener('click', () => {
				router.navigate(`/item_id/${item.item_id}`)
			})
			td_first.style.cursor = "pointer";
			DragUtil.setup_drag(td_first, item);
			DragUtil.setup_drop(td_first, {"Parent": [item.title]})

			td_first.appendChild(title_link)
			element.appendChild(td_first)

			// And then the attributes, with editable value views
			this.data!.columns.forEach(attr => {
				let td = document.createElement('td');
				let attr_value_view = new AttributeValueView();
				attr_value_view.init({
					key: attr,
					value: item.attributes![attr]
				})
				attr_value_view.addEventListener('change', async () => {
					await API.item.Attributes.set_value(item.item_id, attr,
						attr_value_view.value
					);
				})
				td.appendChild(attr_value_view)
				element.appendChild(td)
			})

			container.append(element);
		})
	}
}

customElements.define('item-table', ItemTable)

export default ItemTable;