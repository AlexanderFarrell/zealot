type Listener<T> = (state: T) => void

export class Store<T> {
    private state: T
    private listeners: Listener<T>[] = []

    constructor(initial: T) {
        this.state = initial
    }

    get() {
        return this.state
    }

    set(partial: Partial<T>) {
        this.state = {...this.state, ...partial}
        this.listeners.forEach(l => l(this.state))
    }

    subscribe(listener: Listener<T>) {
        this.listeners.push(listener)
    }
}