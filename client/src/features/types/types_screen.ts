import API from "../../api/api";
import { BaseElementEmpty } from "../../shared/base_element";
import { router } from "../router/router";

class TypesScreen extends BaseElementEmpty {
	async render() {
		this.classList.add('center');
		let types = await API.item.Types.get_all();

		this.innerHTML = `<h1>Types</h1>
		<table><thead><td>Item Type</td><td>Description</td><td>Built-In</td></thead></table>
		`;

		types.forEach(t => {
			let element = document.createElement('tr')
			element.innerHTML = `
			<td>${t.name}</td>
			<td>${t.description}</td>
			<td>${t.is_system ? 'Yes' : 'No'}</td>
			`
			// element.classList.add('box')
			// element.style.display = "block";
			// element.style.padding = "4px";
			// element.style.cursor = "pointer";
			// element.innerHTML = t.name + ' - ' + t.description;
			element.addEventListener('click', () => {
				router.navigate(`/types/${t.name}`)
			})
			this.querySelector('table')!.appendChild(element)
		})
	}
}

customElements.define('types-screen', TypesScreen);

export default TypesScreen;