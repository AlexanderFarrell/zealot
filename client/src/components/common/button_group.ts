import BaseElement from "./base_element";

export class ButtonDef {
    public Icon: string;
    public Title: string;
    public On: Function;

    public constructor(icon: string, title: string, on: Function) {
        this.Icon = icon;
        this.Title = title;
        this.On = on;
    }
}

export class ButtonGroup extends BaseElement<ButtonDef[]> {
    render() {
        this.innerHTML = "";
        let buttons = this.data;
        buttons?.forEach(buttonDef => {
            let view = document.createElement('button') as HTMLButtonElement;
            view.classList.add('button-group-button');
            view.innerHTML = `<img style="width: 2em" src="${buttonDef.Icon}">`;
            view.title = buttonDef.Title;
            view.addEventListener('click', () => {
                buttonDef.On();
            })
            this.appendChild(view)
        })
    }

    setDirection(isBlock = false) {
         
    }
}

customElements.define('button-group', ButtonGroup);

export default ButtonGroup;