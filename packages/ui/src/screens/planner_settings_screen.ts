import { BaseElementEmpty } from '@websoil/engine';

export class PlannerSettingsScreen extends BaseElementEmpty {
    render() {
        this.className = 'planner-settings-screen';
        this.innerHTML = '';

        const heading = document.createElement('h2');
        heading.textContent = 'Planner Settings';

        const note = document.createElement('p');
        note.className = 'tool-muted';
        note.textContent = 'Planner settings are not yet configured.';

        this.append(heading, note);
    }
}

if (!customElements.get('planner-settings-screen')) {
    customElements.define('planner-settings-screen', PlannerSettingsScreen);
}
