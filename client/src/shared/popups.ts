import BaseElement from "./base_element";

export type ErrorType = "error" | "warning" | "note";

interface PopupInfo {
	kind: ErrorType,
	text: string,
	time_to_live: number,
}

class Popup extends BaseElement<PopupInfo> {
	render() {
		this.innerText = this.data!.text;
		this.classList.add(this.data!.kind);

		setTimeout(() => {
			this.remove();
		}, this.data!.time_to_live * 1000);
	}
}

class Popups {
	private static container: HTMLDivElement | null = null;

	private static init() {
		// Create element
		Popups.container = document.createElement('div');
		Popups.container!.id = "popup_container";
		document.body.appendChild(Popups.container!);
	}

	static add(text: string, kind: ErrorType = "note", time_to_live: number = 5) {
		// Lazy init
		if (!Popups.container) {
			Popups.init()
		}

		Popups.container?.appendChild(new Popup().init({
			kind,
			text,
			time_to_live
		}))
	}

	static add_error(text: string) {
		Popups.add(text, 'error', 10);
	}

	static add_warning(text: string) {
		Popups.add(text, 'warning', 10);
	}
}

customElements.define('popup-', Popup);

export default Popups;