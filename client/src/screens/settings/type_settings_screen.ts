class TypeSettingsScreen extends HTMLElement {
    connectedCallback() {
        this.innerHTML = "<h1>Type Settings</h1>"
    }

    disconnectedCallback() {

    }
}

customElements.define('type-settings-screen', TypeSettingsScreen);

export default TypeSettingsScreen;