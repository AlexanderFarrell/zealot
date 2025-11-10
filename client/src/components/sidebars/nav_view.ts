

class NavView extends HTMLElement {
    connectedCallback() {
        this.textContent = "Nav View!"
    }
}

customElements.define('nav-view', NavView)

export default NavView;