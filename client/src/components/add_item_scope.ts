import API from "../api/api";
import BaseElement from "./common/base_element";
import AddIcon from "../assets/icon/add.svg";

class AddItemScoped extends BaseElement<Record<string, any>> {
    private events: Set<Function> = new Set();

    render() {
        let attributes = this.data!;

        this.innerHTML = `<form style="display: grid; grid-template-columns: 1fr auto; margin-bottom: 10px">
            <input name="title" type="text" placeholder="Add New Item..."></input>
            <button type="submit"><img style="width: 1em" src="${AddIcon}"></button>
        </form>`

        let form = this.querySelector('form')! as HTMLFormElement;
        let title = (this.querySelector('[name="title"]')! as HTMLInputElement);

        form.addEventListener('submit', async (e: SubmitEvent) => {
            e.preventDefault();
            let data = new FormData(form);
            await API.item.add({
                item_id: -1,
                title: data.get('title') as string,
                content: '',
                attributes: attributes
            })
            title.value = "";

            this.dispatchEvent(new Event('change', {bubbles: true}));
            title.focus();
        })
    }

    listen_on_submit(func: Function) {
        this.events.add(func)
    }
}

customElements.define('add-item-scoped', AddItemScoped)

export default AddItemScoped;