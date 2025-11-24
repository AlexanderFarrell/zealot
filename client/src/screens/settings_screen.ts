class SettingsScreen extends HTMLElement {
    connectedCallback() {
        this.innerHTML = "<h1>Settings</h1>"
    }

    disconnectedCallback() {
    }
}

customElements.define('settings-screen', SettingsScreen)

export default SettingsScreen;