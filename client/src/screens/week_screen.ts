class WeeklyPlannerScreen extends HTMLElement {
    connectedCallback() {
        this.innerHTML = "<h1>Weekly Planner</h1>"
    }

    disconnectedCallback() {
    }
}

customElements.define('weekly-planner-screen', WeeklyPlannerScreen)

export default WeeklyPlannerScreen;