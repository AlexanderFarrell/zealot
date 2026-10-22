import { BaseElementEmpty } from '@websoil/engine';
import { buildEmptyState, buildSectionHeader } from '../common/settings_layout';

export class PlannerSettingsScreen extends BaseElementEmpty {
    render() {
        this.className = 'planner-settings-screen';
        this.innerHTML = '';

        const shell = document.createElement('div');
        shell.className = 'settings-section-shell';
        shell.append(
            buildSectionHeader('Planner', 'Planner preferences.').element,
            buildEmptyState('Nothing to configure yet', 'Daily, weekly, monthly and annual planner options will live here.'),
        );
        this.appendChild(shell);
    }
}

if (!customElements.get('planner-settings-screen')) {
    customElements.define('planner-settings-screen', PlannerSettingsScreen);
}
