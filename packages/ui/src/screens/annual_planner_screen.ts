import { BaseElementEmpty } from '@websoil/engine';

export class AnnualPlannerScreen extends BaseElementEmpty {
    async render() {
        this.innerHTML = '<p>TODO: AnnualPlannerScreen</p>';
    }

    init(year: string): this {
        // TODO: implement with year
        void year;
        return this;
    }
}

customElements.define('annual-planner-screen', AnnualPlannerScreen);
