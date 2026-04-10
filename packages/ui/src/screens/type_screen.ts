import { BaseElementEmpty } from '@websoil/engine';

export class TypeScreen extends BaseElementEmpty {
    async render() {
        this.innerHTML = '<p>TODO: TypeScreen</p>';
    }

    init(title: string): this {
        // TODO: implement with type title
        void title;
        return this;
    }
}

customElements.define('type-screen', TypeScreen);
