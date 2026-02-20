class RulesScreen extends HTMLElement {
    connectedCallback() {
        this.innerHTML = "<h1>Rules</h1>"
    }

    disconnectedCallback() {
    }
}

customElements.define('rules-screen', RulesScreen)

export default RulesScreen;