import type { Item } from "../api/item";
import { router } from "../core/router";
import BaseElement, { BaseAPIElement } from "./common/base_element";

interface ItemTableInfo {
	items: Item[],
	columns: string[]
}

class ItemTable extends BaseElement<ItemTableInfo> {
	render() {
		this.innerHTML = `
		<table name="items"></div>
		`

		let container = this.querySelector('[name="items"]')! as HTMLTableElement;

		let headings = ['Name', ...this.data!.columns];

		container.innerHTML = 
		"<thead>" +
		headings.map(i => {
			return `<td>${i}</td>`
		}).join("")
		+ "</thead>"

		this.data!.items.forEach(item => {
			let element: HTMLTableRowElement = document.createElement('tr');

			let values = [item.title, ...this.data!.columns.map(i => item.attributes![i])]

			element.innerHTML = 
				values.map(i => {
					return `<td>${i || ""}</td>`
				}).join("")
			element.addEventListener('click', () => {
				router.navigate(`/item/${item.title}`)
			})

			container.append(element);
		})
	}
}

customElements.define('item-table', ItemTable)

export default ItemTable;