class MonthlyPlannerScreen extends HTMLElement {
    connectedCallback() {
        this.innerHTML = "<h1>Monthly Planner</h1>"
    }

    disconnectedCallback() {
    }
}

customElements.define('monthly-planner-screen', MonthlyPlannerScreen)

export default MonthlyPlannerScreen;