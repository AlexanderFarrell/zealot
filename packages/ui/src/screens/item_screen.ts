import { BaseElementEmpty } from '@websoil/engine';

export class ItemScreen extends BaseElementEmpty {
    async render() {
        this.innerHTML = '<p>TODO: ItemScreen</p>';
    }

    loadItem(_title: string): void {
        // TODO: implement
    }

    loadItemById(_id: number): void {
        // TODO: implement
    }
}

customElements.define('item-screen', ItemScreen);
