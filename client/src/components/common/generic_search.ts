import Popups from "../../core/popups";
import BaseElement from "./base_element";

interface GenericSearchInfo<T> {
	on_search: (term: string) => Promise<T[]>,
	on_select: (item: T) => void,
	on_make_view: (item: T) => HTMLElement,
}

export class GenericSearch<T> extends BaseElement<GenericSearchInfo<T>> {
	private results_views: HTMLElement[] = []
	private result_index: number = -1;

	async render() {
		this.innerHTML = `
		<input name="search" type="text" required>
		<div name="generic_results">
		&nbsp;
		</div>
		`

		let results: T[] = [];
		let search_input = this.querySelector('[name="search"]')! as HTMLInputElement;
		let results_view = this.querySelector('[name="generic_results"]')! as HTMLDivElement;

		search_input.addEventListener('input', async () => {
			this.results_views = [];
			try {
				let term = search_input.value;
				results_view.innerHTML = '';
				if (term === "") {
					return;
				}
				results = await this.data!.on_search(search_input.value);

			} catch (e) {
				Popups.add_error(`Error getting results: ${e}`)
				return;
			}

			results.forEach((result, index) => {
				let view = this.data!.on_make_view(result);
				view.addEventListener('click', () => {
					this.data!.on_select(result);
				})
				this.results_views.push(view);
				results_view.appendChild(view);
			})
			if (results.length > 0) {
				this.result_index = 0;
				this.set_selected_result();
			}
		})

		search_input.addEventListener('keydown', (e: KeyboardEvent) => {
			if (e.key === 'Enter') {
				if (results.length > this.result_index && results.length != 0) {
					this.data!.on_select(results[this.result_index]);
				}
			}
			else if (e.key === "ArrowUp") {
				this.result_index--;
				if (this.result_index < 0) {
					this.result_index = 0;
				}
				this.set_selected_result();
			}
			else if (e.key == 'ArrowDown') {
                this.result_index++;
                if (this.result_index >= results.length) {
                    this.result_index = results.length - 1;
                    if (this.result_index < 0) {
                        this.result_index = 0;
                    }
                }
                this.set_selected_result();
            }
		})
	}

	focus() {
		let input = this.querySelector('[name="search"]')! as HTMLInputElement;
		input.focus();
	}

	set_selected_result() {
		this.results_views.forEach(rv => {
			rv.classList.remove('selected');
		})
		if (this.result_index === -1) {
			return;
		}
		this.results_views[this.result_index].classList.add('selected');
	}
}