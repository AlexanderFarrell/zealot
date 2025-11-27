class DataSettingsScreen extends HTMLElement {
    connectedCallback() {
        this.innerHTML = "<h1>Data Settings</h1>"
    }

    disconnectedCallback() {

    }
}

customElements.define('data-settings-screen', DataSettingsScreen);

export default DataSettingsScreen;