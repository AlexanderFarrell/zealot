import { BaseElementEmpty } from '@websoil/engine';

export class DataSettingsScreen extends BaseElementEmpty {
    render() {
        this.className = 'data-settings-screen';
        this.innerHTML = '';

        const heading = document.createElement('h2');
        heading.textContent = 'Data Settings';

        const note = document.createElement('p');
        note.className = 'tool-muted';
        note.textContent = 'Data import and export are not yet configured.';

        this.append(heading, note);
    }
}

if (!customElements.get('data-settings-screen')) {
    customElements.define('data-settings-screen', DataSettingsScreen);
}
