import type { Item } from "../api/item";
import { router } from "../core/router";


class PlanView extends HTMLElement {
    private _item: Item | null = null;

    public get item(): Item | null {
        return this._item;
    }

    public set item(value: Item) {
        this._item = value;
        this.render();
    }

    async render() {
        this.innerHTML = `
        <div name="icon"></div>
        <div name="title"></div>
        <div name="status"></div>
        <div name="priority"></div>
        `

        let icon_view = this.querySelector('[name="icon"]')! as HTMLElement;
        let title_view = this.querySelector('[name="title"]')! as HTMLElement;
        let status_view = this.querySelector('[name="status"]')! as HTMLElement;
        let priority_view = this.querySelector('[name="priority"]')! as HTMLElement;

        title_view.innerText = this.item!.title;

        let set_attr = (view: HTMLElement, attribute: string) => {
            if (this.item!.attributes![attribute] != null) {
                view.innerText = this.item!.attributes![attribute];
            } else {
                view.innerHTML = "&nbsp;"
            }
        }

        set_attr(icon_view, "Icon")
        set_attr(status_view, 'Status')
        set_attr(priority_view, "Priority")

        this.addEventListener('click', () => {
            router.navigate(`/item/${encodeURIComponent(this.item!.title)}`);
        })
    }
}

customElements.define('plan-view', PlanView)

export default PlanView;