import API from "../api/api";
import { BaseElementEmpty } from "../components/common/base_element";
import { router } from "../core/router";

class TypesScreen extends BaseElementEmpty {
	async render() {
		this.classList.add('center');
		let types = await API.item.Types.get_all();

		this.innerHTML = "<h1>Types</h1>";

		types.forEach(t => {
			let element = document.createElement('a')
			element.innerHTML = t.name + ' - ' + t.description;
			element.addEventListener('click', () => {
				router.navigate(`/types/${t.name}`)
			})
			this.appendChild(element)
		})
	}
}

customElements.define('types-screen', TypesScreen);

export default TypesScreen;