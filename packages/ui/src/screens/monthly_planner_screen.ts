import { BaseElementEmpty } from '@websoil/engine';

export class MonthlyPlannerScreen extends BaseElementEmpty {
    async render() {
        this.innerHTML = '<p>TODO: MonthlyPlannerScreen</p>';
    }

    init(date: string): this {
        // TODO: implement with date (yyyy-MM)
        void date;
        return this;
    }
}

customElements.define('monthly-planner-screen', MonthlyPlannerScreen);
