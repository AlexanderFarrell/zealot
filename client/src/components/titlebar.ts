

class TitleBar extends HTMLElement {
    connectedCallback() {
        this.textContent = "Title!"
    }
}

customElements.define('title-bar', TitleBar)

export default TitleBar;