class AnalysisScreen extends HTMLElement {
    connectedCallback() {
        this.innerHTML = "<h1>Analysis</h1>"
    }

    disconnectedCallback() {
    }
}

customElements.define('analysis-screen', AnalysisScreen)

export default AnalysisScreen;