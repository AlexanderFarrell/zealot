import { BaseElementEmpty } from "../components/common/base_element";

class AnalysisScreen extends BaseElementEmpty {
    render() {
        this.innerHTML = "<h1>Analysis</h1>"
    }
}

customElements.define('analysis-screen', AnalysisScreen)

export default AnalysisScreen;