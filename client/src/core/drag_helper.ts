import API from "../api/api";
import type { Item } from "../api/item";
import Popups from "./popups";
import { ToConjunctionList } from "./string_helper";

class DragUtil {
	public static item: Item | null = null;

	private constructor() {

	}

	public static setup_drag(element: HTMLElement, item: Item) {
		element.setAttribute("draggable", "true");
		element.addEventListener('dragstart', (e: DragEvent) => {
			DragUtil.item = item;
			// TODO: Modfy this later to have full item
			if (e.dataTransfer) {
				e.dataTransfer.setData('text/plain', JSON.stringify(item));
				e.dataTransfer.effectAllowed = "move";
			}
		})

		element.addEventListener('dragend', () => {
			DragUtil.item = null;
		})
	}

	public static setup_drop(element: HTMLElement, attributes: Record<string, any>) {
		element.addEventListener('dragover', (e: DragEvent) => {
			e.preventDefault();
		})
		element.addEventListener('drop', async (e: DragEvent) => {
			e.preventDefault();
			if (!DragUtil.item) {
				return;
			}
			try {
				let item = DragUtil.item;
				for (let key of Object.keys(attributes)) {
					await API.item.Attributes.set_value(item.item_id,
						key,
						attributes[key]
					)
				}
				Popups.add(
					`Set ${ToConjunctionList(Object.entries(attributes).map(e => {
						return `${e[0]} to ${e[1]}`
					}), "and")} for ${item.title}`
				)

			} catch (e) {
				// TODO
				console.error(e)
			} finally {
				DragUtil.item = null;
			}
		})
	}
}

export default DragUtil;