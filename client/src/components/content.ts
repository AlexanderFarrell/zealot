

class Content extends HTMLElement {
    connectedCallback() {
        this.innerHTML = "<item-screen></item-screen>"
        // this.innerHTML = "<>"
    }
}

customElements.define('content-', Content)

export default Content;