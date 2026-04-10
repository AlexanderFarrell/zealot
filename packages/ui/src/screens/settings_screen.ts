import { BaseElementEmpty } from '@websoil/engine';

export class SettingsScreen extends BaseElementEmpty {
    async render() {
        this.innerHTML = '<p>TODO: SettingsScreen</p>';
    }

    switchScreen(_section: string): void {
        // TODO: implement
    }
}

customElements.define('settings-screen', SettingsScreen);
