import { BaseElementEmpty, getNavigator, type SettingsSection } from '@websoil/engine';
import { TypeSettingsScreen } from './type_settings_screen';

export class SettingsScreen extends BaseElementEmpty {
    private section: SettingsSection = 'attributes';

    async render() {
        this.className = 'settings-screen';
        this.innerHTML = '';

        const shell = document.createElement('div');
        shell.className = 'settings-screen-shell';

        const title = document.createElement('h1');
        title.textContent = 'Settings';

        const nav = document.createElement('div');
        nav.className = 'settings-section-nav';

        const sections: Array<{ key: SettingsSection; label: string }> = [
            { key: 'attributes', label: 'Attributes' },
            { key: 'types', label: 'Types' },
            { key: 'planner', label: 'Planner' },
            { key: 'wiki', label: 'Wiki' },
            { key: 'data', label: 'Data' },
            { key: 'user', label: 'User' },
        ];

        sections.forEach(({ key, label }) => {
            const button = document.createElement('button');
            button.type = 'button';
            button.textContent = label;
            button.className = key === this.section ? 'is-active' : '';
            button.addEventListener('click', () => {
                getNavigator().openSettings(key);
            });
            nav.appendChild(button);
        });

        const body = document.createElement('div');
        body.className = 'settings-screen-body';
        if (this.section === 'types') {
            body.appendChild(new TypeSettingsScreen());
        } else {
            const message = document.createElement('p');
            message.className = 'tool-muted';
            message.textContent = `The ${this.section} settings screen is not implemented yet.`;
            body.appendChild(message);
        }

        shell.append(title, nav, body);
        this.appendChild(shell);
    }

    switchScreen(section: string): void {
        this.section = this.isSettingsSection(section) ? section : 'attributes';
        if (this.isConnected) {
            void this.render();
        }
    }

    private isSettingsSection(section: string): section is SettingsSection {
        return ['attributes', 'types', 'planner', 'wiki', 'data', 'user'].includes(section);
    }
}

customElements.define('settings-screen', SettingsScreen);
