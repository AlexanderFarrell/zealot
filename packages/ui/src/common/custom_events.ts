export type OnBackEvent = CustomEvent<void>;

declare global {
    interface HTMLElementEventMap {
        "back": OnBackEvent,
    }
}