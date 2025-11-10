

class Content extends HTMLElement {
    connectedCallback() {
        this.textContent = "Content!"
    }
}

customElements.define('content-', Content)

export default Content;