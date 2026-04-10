import { BaseElementEmpty } from '@websoil/engine';

export class DailyPlannerScreen extends BaseElementEmpty {
    async render() {
        this.innerHTML = '<p>TODO: DailyPlannerScreen</p>';
    }

    init(date: string): this {
        // TODO: implement with date (yyyy-MM-dd)
        void date;
        return this;
    }
}

customElements.define('daily-planner-screen', DailyPlannerScreen);
