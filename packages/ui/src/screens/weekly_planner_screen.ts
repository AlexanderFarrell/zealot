import { BaseElementEmpty } from '@websoil/engine';

export class WeeklyPlannerScreen extends BaseElementEmpty {
    async render() {
        this.innerHTML = '<p>TODO: WeeklyPlannerScreen</p>';
    }

    init(date: string): this {
        // TODO: implement with date (YYYY-Www)
        void date;
        return this;
    }
}

customElements.define('weekly-planner-screen', WeeklyPlannerScreen);
