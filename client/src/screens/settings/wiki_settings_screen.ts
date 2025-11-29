class WikiSettingsScreen extends HTMLElement {
    connectedCallback() {
        this.innerHTML = "<h1>Wiki Settings</h1>"
    }

    disconnectedCallback() {

    }
}

customElements.define('wiki-settings-screen', WikiSettingsScreen);

export default WikiSettingsScreen;