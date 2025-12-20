import BaseElement from "../common/base_element";
import DeleteIcon from "../../assets/icon/close.svg";
import AddIcon from "../../assets/icon/add.svg";
import { get_kind_for_key, type AttributeKind } from "../../api/attribute_kind";
import API from "../../api/api";
import ItemAPI from "../../api/item";
import ChipsInput from "../common/chips_input";

interface AttributeItem {
	item_id: number,
	key: string,
	value: any,
	is_new: boolean
}

type AttrAddEvent = CustomEvent<{attr: AttributeItem}>;
type AttrRemoveEvent = CustomEvent<{attr: AttributeItem}>;
type AttrRenameEvent = CustomEvent<{old_key: string, new_key: string}>;

declare global {
	interface HTMLElementEventMap {
		'attr-add': AttrAddEvent,
		'attr-remove': AttrRemoveEvent,
		'attr-rename': AttrRenameEvent
	}
}

class AttributeItemView extends BaseElement<AttributeItem> {
	async render() {
		let attr = this.data!;
		this.classList.add('attribute')

		this.innerHTML = `
			<input type="text" name="key" tab-index="1"
			 	list="attribute_kind_suggestions" required>
			<span name="value_view"></span>
			<button type="submit">
				<img src="${(attr.is_new) ? AddIcon : DeleteIcon}" 
					style="width: 1em;">
			</button>
		`

        let key_input: HTMLInputElement = 
			this.querySelector('[name="key"]')!;
		let button_submit: HTMLButtonElement =
			this.querySelector('[type="submit"]')!;
		this.setup_value_view();

		key_input.value = attr.key;

		key_input.addEventListener('change', async () => {
			let old_key = attr.key;
			let new_key = key_input.value;
			attr.key = new_key;
			if (attr.is_new) {
				this.setup_value_view();
				return;
			} else {
				await ItemAPI.Attributes.rename(attr.item_id, 
					old_key,
					new_key
				);
				this.dispatchEvent(new CustomEvent('attr-rename', {
					detail: {old_key,new_key}
				}));
				this.render();
			}
		})
		key_input.addEventListener('keydown', (e) => {
			console.log(e.key)
			if (e.key == "Tab") {
				setTimeout(() => {
					let v: HTMLInputElement | HTMLSelectElement | ChipsInput = this.querySelector('[name="value"]')!;
					v.focus()
				}, 50)
			}
		})
		button_submit.addEventListener('click', async () => {
			if (attr.is_new) {
				await API.item.Attributes.set_value(attr.item_id, attr.key, attr.value);
				this.dispatchEvent(new CustomEvent('attr-add', {
					detail: {attr}
				}))
			} else {
				await API.item.Attributes.remove(attr.item_id, key_input.value);
				this.dispatchEvent(new CustomEvent('attr-remove', {
					detail: {attr}
				}))
				this.remove();
			}

		})
	}

	async setup_value_view() {
		let attr = this.data!;
		let kind = await get_kind_for_key(attr.key);

		let value_view: HTMLSpanElement = 
			this.querySelector('[name="value_view"]')!;
		value_view.innerHTML = "";

		let value_input: HTMLSelectElement | HTMLInputElement |
			ChipsInput | null = null;

		if (kind == undefined || kind.base_type == "text") {
			// Text or default 
			value_input = document.createElement('input')
			value_input.type = 'text';
		}
		else if (kind.base_type == 'integer' || kind.base_type == 'decimal') {
			// Any number type
			value_input = document.createElement('input');
			value_input.type = 'number';
			if (kind.base_type == "integer") {
				value_input.step = '1';
			}
			if (kind.config && kind.config.min != null) {
				value_input.min = kind.config.min;
			}
			if (kind.config && kind.config.max != null) {
				value_input.max = kind.config.max;
			}
		}
		else if (kind.base_type == 'date') {
			value_input = document.createElement('input');
			value_input.type = 'date';
		}
		else if (kind.base_type == 'week') {
			value_input = document.createElement('input');
			value_input.type = 'week';
		}
		else if (kind.base_type == 'dropdown') {
			value_input = document.createElement('select');
			// Go through config and get all types
			if (kind.config && kind.config.values) {
				kind.config.values.forEach((v: string) => {
					value_input!.innerHTML += `<option type="${v}">${v}</option>`
				});
			}
		}
		else if (kind.base_type == 'boolean') {
			value_input = document.createElement('input');
			value_input.type = 'checkbox';
		}
		else if (kind.base_type == 'list') {
			value_input = new ChipsInput();
		}
		else {
			value_view.innerText = "Error: Unknown Type";
			return;
		}

		// Set value
		if (kind?.base_type == 'date') {
			value_input!.value = attr.value.substring(0, 10)
		} else {
			value_input!.value = attr.value;
		}

		value_view.appendChild(value_input!);
		value_input.setAttribute('name', 'value');

		value_input?.addEventListener('change', async () => {
			attr.value = value_input!.value;
			if (attr.item_id) {
				await API.item.Attributes.set_value(attr.item_id, attr.key, attr.value);

			}
		})
	}
}

customElements.define('attribute-item-view', AttributeItemView)

export default AttributeItemView;