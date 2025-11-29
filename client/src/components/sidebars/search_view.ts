import ItemAPI from "../../api/item";
import { events } from "../../core/events";
import { router } from "../../core/router";
import { switch_item_to } from "../../screens/item_screen";


class SearchView extends HTMLElement {
    public results: Array<any> = [];
    public result_views: Array<HTMLElement> = [];
    public result_index = 0;

    connectedCallback() {
        let container = document.createElement('div')

        let search_bar = document.createElement('input');
        search_bar.type = 'search';
        search_bar.addEventListener('input', async () => {
            try {
                let term = search_bar.value
                container.innerHTML = '';
                if (term.length == 0) {
                    return;
                }
                this.results = await ItemAPI.search(term) as Array<any>;
                this.result_views = [];
                this.result_index = 0;
                this.results.forEach((result, index) => {
                    let resultView = document.createElement('div')
                    resultView.classList.add('result_view')
                    resultView.innerText = result.title;
                    resultView.addEventListener('click', () => {

                        router.navigate(`/item/${result.title}`)
                        this.result_index = index;
                        this.set_selected_result();
                    })
                    container.appendChild(resultView)
                    this.result_views.push(resultView);
                })
                this.set_selected_result();
            }
            catch (e) {
                console.error(e)
                container.innerHTML = `<error>Error getting items</error>`
            }
        })
        search_bar.addEventListener('keydown', (e: KeyboardEvent) => {
            if (e.key == 'Enter') {
                if (this.results.length > this.result_index && this.results.length != 0) {
                    router.navigate(`/item/${this.results[this.result_index].title}`)
                }
            }
            else if (e.key == 'ArrowUp') {
                this.result_index--;
                if (this.result_index < 0) {
                    this.result_index = 0;
                }
                this.set_selected_result();
            }
            else if (e.key == 'ArrowDown') {
                this.result_index++;
                if (this.result_index >= this.results.length) {
                    this.result_index = this.results.length - 1;
                    if (this.result_index < 0) {
                        this.result_index = 0;
                    }
                }
                this.set_selected_result();
            }
        })

        this.appendChild(search_bar)
        this.appendChild(container);

        search_bar.focus();
    }

    set_selected_result() {
        this.result_views.forEach(rv => {
            rv.classList.remove('selected')
        })

        this.result_views[this.result_index].classList.add('selected');
    }
}

customElements.define('search-view', SearchView)

export default SearchView;