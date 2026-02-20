class PlannerSettingsScreen extends HTMLElement {
    connectedCallback() {
        this.innerHTML = "<h1>Planner Settings</h1>"
    }

    disconnectedCallback() {

    }
}

customElements.define('planner-settings-screen', PlannerSettingsScreen);

export default PlannerSettingsScreen;