

abstract class BaseElement<T extends any> extends HTMLElement {
    private _data: T | null = null;
    public get data(): T | null {
        return this._data;
    }

    public set data(data: T) {
        this._data = data;
        this.render();
    }

    public init(data: T): BaseElement<T> {
        this.data = data;
        return this
    }

    public abstract render(): any;
}

function BaseElementMixin<TData, J extends HTMLElement>(base: TData) {

}

export default BaseElement;