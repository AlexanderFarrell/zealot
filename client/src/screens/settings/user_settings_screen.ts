class UserSettingsScreen extends HTMLElement {
    connectedCallback() {
        this.innerHTML = "<h1>User Settings</h1>"
    }

    disconnectedCallback() {

    }
}

customElements.define('user-settings-screen', UserSettingsScreen);

export default UserSettingsScreen;