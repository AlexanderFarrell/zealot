import { BaseElementEmpty } from '@websoil/engine';
import { buildEmptyState, buildSectionHeader } from '../common/settings_layout';

export class WikiSettingsScreen extends BaseElementEmpty {
    render() {
        this.className = 'wiki-settings-screen';
        this.innerHTML = '';

        const shell = document.createElement('div');
        shell.className = 'settings-section-shell';
        shell.append(
            buildSectionHeader('Wiki', 'Wiki preferences.').element,
            buildEmptyState('Nothing to configure yet', 'Options for how items and links behave will live here.'),
        );
        this.appendChild(shell);
    }
}

if (!customElements.get('wiki-settings-screen')) {
    customElements.define('wiki-settings-screen', WikiSettingsScreen);
}
