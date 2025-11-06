

class IconButton extends HTMLButtonElement {
    public image_src: string;
    public title: string;
    public on: Function;

    constructor(image_src: string, title: string, on: Function) {
        super()
        this.image_src = image_src;
        this.title = title;
        this.on = on;
    }

    connectedCallback() {
    }
}