import { BaseElementEmpty } from '@websoil/engine';

export class MediaScreen extends BaseElementEmpty {
    async render() {
        this.innerHTML = '<p>TODO: MediaScreen</p>';
    }

    init(path: string): this {
        // TODO: implement with path
        void path;
        return this;
    }
}

customElements.define('media-screen', MediaScreen);
