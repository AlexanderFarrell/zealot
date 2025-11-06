

class TitleBar extends HTMLElement {
    connectedCallback() {
        this.textContent = "Hello!"
    }
}

customElements.define('title-bar', TitleBar)

export default TitleBar;