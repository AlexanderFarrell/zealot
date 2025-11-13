class DailyPlannerScreen extends HTMLElement {
    connectedCallback() {
        this.innerHTML = "<h1>Daily Planner</h1>"
    }

    disconnectedCallback() {
    }
}

customElements.define('daily-planner-screen', DailyPlannerScreen)

export default DailyPlannerScreen;