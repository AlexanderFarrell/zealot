

class IconButton extends HTMLElement {
    public image_src: string;

    constructor(image_src: string, title: string, on: Function) {
        super()
        this.image_src = image_src;
        this.title = title;
        this.addEventListener('click', () => {
            on();
        });
    }

    connectedCallback() {
        this.innerHTML = `<img class="icon" src="${this.image_src}">`;
    }
}

customElements.define('icon-button', IconButton);

export default IconButton;