
export abstract class BaseAPIElement<T extends any> extends HTMLElement {
    private _data: T | null = null;

    protected abstract get_data(): Promise<T>;
    protected abstract on_render(): any;
    public async refresh() {
        this._data = await this.get_data();
        this.on_render();
    }
}

abstract class BaseElement<T extends any> extends HTMLElement {
    private _data: T | null = null;
    protected is_rendered = false;
    public get data(): T | null {
        return this._data;
    }

    public set data(data: T) {
        this._data = data;
        this.render();
        this.is_rendered = true;
    }

    public init(data: T): BaseElement<T> {
        this.data = data;
        return this
    }

    public abstract render(): any;
}

export abstract class BaseElementEmpty extends HTMLElement {
    connectedCallback() {
        this.render()
    }

    public abstract render(): any
}

function BaseElementMixin<TData, J extends HTMLElement>(base: TData) {

}

export function GenerateSelectHTML(name: string, value: string, values: Array<string>, innerTexts: Array<string> = values) {
    if (values.length == 0) {
        throw new Error(`Values must be greater than 0 for Select: ${name}`)
    }

    if (values.length != innerTexts.length) {
        throw new Error(`We must have an equal number of inner texts to values for Select: ${name}`)
    }

    let html = `<select name="${name}" value="${value}">`
    for (let i = 0; i < values.length; i++) {
        html += `<option value="${values[i]}">${innerTexts[i]}</option>`;
    }
    html += "</select>"
    return html
}


// interface FormInputDef {
//     name: string,
//     type: string,

// }

// export function GenerateBasicForm(id: string, fields: FormInputDef[]): HTMLFormElement {
//     let form = document.createElement('form')
//     form.classList.add('box')
//     form.id = id;

//     fields.forEach(field => {
//         switch (field.type) {

//         }
//     })

//     return from
// }

export default BaseElement;