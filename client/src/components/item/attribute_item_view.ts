import BaseElement from "../common/base_element";
import DeleteIcon from "../../assets/icon/close.svg";
import AddIcon from "../../assets/icon/add.svg";
import { get_kind_for_key, type AttributeKind } from "../../api/attribute_kind";
import API from "../../api/api";
import ItemAPI from "../../api/item";
import ChipsInput from "../common/chips_input";
import { router } from "../../core/router";

interface AttributeItem {
	key: string,
	value: any,
}

type AttrAddEvent = CustomEvent<{attr: AttributeItem}>;
type AttrRemoveEvent = CustomEvent<{attr: AttributeItem}>;
type AttrRenameEvent = CustomEvent<{old_key: string, new_key: string}>;
type AttrChangeEvent = CustomEvent<{key: string, new_value: any}>

declare global {
	interface HTMLElementEventMap {
		'attr-add': AttrAddEvent,
		'attr-remove': AttrRemoveEvent,
		'attr-rename': AttrRenameEvent,
		'attr-value-change': AttrChangeEvent
	}
}

export class AttributeValueView extends BaseElement<AttributeItem> {
	async render() {
		let attr = this.data!;
		let kind = await get_kind_for_key(attr.key);

		this.innerHTML = "";

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
			(value_input as HTMLInputElement).checked = 
				attr.value === true || attr.value === 1;
		}
		else if (kind.base_type == 'list') {
			value_input = new ChipsInput();

			// Hack need to fix later
			if (attr.key == "Parent") {
				let on: (i: string) => void = (i) => {
					router.navigate(`/item/${i}`)
				}
				// @ts-ignore
				(value_input as ChipsInput).OnClickItem = on;
			}
		}
		else {
			this.innerText = "Error: Unknown Type";
			return;
		}

		// Set value
		if (kind?.base_type == 'date') {
			value_input!.value = attr.value.substring(0, 10)
		} else {
			value_input!.value = attr.value;
		}

		this.appendChild(value_input!);
		value_input.setAttribute('name', 'value');

		value_input!.addEventListener('change', async () => {
			if (kind && kind!.base_type == "boolean") {
				this.data!.value = (value_input as HTMLInputElement).checked;
			} else {
				this.data!.value = value_input!.value;
			}
			this.dispatchEvent(new Event('change'));
		})
	}

	get key(): string {
		return this.data!.key;
	}

	get value(): any {
		return this.data!.value;
	}

	focus() {
		this.querySelector('input')?.focus();
	}
}

class AttributeItemView extends BaseElement<AttributeItem> {
	async render() {
		let attr = this.data!;
		this.classList.add('attribute')

		this.innerHTML = `
			<input type="text" name="key" tab-index="1"
			 	list="attribute_kind_suggestions" required>
			<attribute-value-view></attribute-value-view>
		`

        let key_input: HTMLInputElement = 
			this.querySelector('[name="key"]')!;
		let value_view: AttributeValueView =
			this.querySelector('attribute-value-view')!;
		
		value_view.init(this.data!);

		key_input.value = attr.key;

		key_input.addEventListener('change', async () => {
			let old_key = attr.key;
			let new_key = key_input.value;
			attr.key = new_key;
			this.dispatchEvent(new CustomEvent('attr-rename', {
				detail: {old_key, new_key}
			}))
		})
		key_input.addEventListener('keydown', (e) => {
			if (e.key == "Tab") {
				return;
			}
			setTimeout(() => {
				value_view.data!.key = key_input.value;
				value_view.render()
			}, 30)
		})
		value_view.addEventListener('change', async () => {
			attr.value = value_view.value;
			this.dispatchEvent(new CustomEvent('attr-value-change', {
				detail: {
					key: attr.key,
					new_value: attr.value
				}
			}))
		})
	}

	get key() {
		return this.data!.key;
	}

	get value() {
		return this.data!.value;
	}
}

customElements.define('attribute-item-view', AttributeItemView)
customElements.define('attribute-value-view', AttributeValueView)

export default AttributeItemView;