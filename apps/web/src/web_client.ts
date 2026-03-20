import { BaseElementEmpty } from "@websoil/engine";

// Just for the UI itself. Much should come from shared components.
class ZealotWebClient extends BaseElementEmpty {
    render() {
        // A lot should come from shared components.
        this.innerHTML = `
        <title-bar>
        </title-bar>
        <main>
            <side-buttons class="desktop_only"></side-buttons>
            <side-bar id="left"
        </main>
        `
    }
}

customElements.define('zealot-web-client', ZealotWebClient);

export default ZealotWebClient;