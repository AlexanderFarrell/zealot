class MediaScreen extends HTMLElement {
    connectedCallback() {
        this.innerHTML = "<h1>Media</h1>"
    }

    disconnectedCallback() {
    }
}

customElements.define('media-screen', MediaScreen)

export default MediaScreen;