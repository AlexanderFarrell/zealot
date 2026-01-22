import BaseElement, { BaseElementEmpty } from "./common/base_element";
import commands, { type UICommand } from "../core/command_runner";
import { GenericSearch } from "./common/generic_search";

class CommandSearch extends GenericSearch<UICommand> {}

export class CommandModal extends BaseElementEmpty {
	render() {
		this.classList.add('modal_background')

		this.innerHTML = `
		<div class="inner_window">
			<command-search></command-search>
		</div>
		`

		let inner_window = this.querySelector('.inner_window')! as HTMLDivElement;
		

		inner_window.addEventListener('click', (e: MouseEvent) => {
			e.stopPropagation();
		})
		this.addEventListener('click', (e: MouseEvent) => {
			this.remove();
		})

		let search_view = this.querySelector('command-search')! as CommandSearch;
		search_view.init({
			on_search: async (term: string) => {
				return commands.search_commands(term);
			},
			on_select: (item: UICommand) => {
				commands.run(item.name);
				this.remove();
			},
			on_make_view: (item: UICommand) => {
				let div = document.createElement('div');
				div.classList.add('command_result');
				div.innerHTML = `<div>${item.name}</div> <div>${item.hotkeys.map(h => h.toString()).join(" or ")}</div>`;
				return div;
			}
		})

		search_view.focus();
	}
}

customElements.define('command-search', CommandSearch);
customElements.define('command-modal', CommandModal);

export default CommandModal;