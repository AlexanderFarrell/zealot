class AnnualPlannerScreen extends HTMLElement {
    connectedCallback() {
        this.innerHTML = "<h1>Annual Planner</h1>"
    }

    disconnectedCallback() {
    }
}

customElements.define('annual-planner-screen', AnnualPlannerScreen)

export default AnnualPlannerScreen;