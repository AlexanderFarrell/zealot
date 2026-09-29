import { BaseElementEmpty } from '@websoil/engine';

export class WikiSettingsScreen extends BaseElementEmpty {
    render() {
        this.className = 'wiki-settings-screen';
        this.innerHTML = '';

        const heading = document.createElement('h2');
        heading.textContent = 'Wiki Settings';

        const note = document.createElement('p');
        note.className = 'tool-muted';
        note.textContent = 'Wiki settings are not yet configured.';

        this.append(heading, note);
    }
}

if (!customElements.get('wiki-settings-screen')) {
    customElements.define('wiki-settings-screen', WikiSettingsScreen);
}
