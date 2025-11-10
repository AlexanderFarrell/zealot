

class SearchView extends HTMLElement {
    connectedCallback() {
        this.innerHTML = '<input type="search">'
    }
}

customElements.define('search-view', SearchView)

export default SearchView;