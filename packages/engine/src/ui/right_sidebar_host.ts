export interface SideViewMeta {
    /** Name shown on the phone pane strip, e.g. "Related" or "Repeats". */
    label: string;
    count?: number | null;
}

export interface RightSidebarHost {
    setContent(el: HTMLElement | null, meta?: SideViewMeta): void;
    /** Update the count on the phone pane strip once it is known. */
    setCount(count: number | null): void;
}

let _host: RightSidebarHost | null = null;

export function setRightSidebarHost(host: RightSidebarHost): void {
    _host = host;
}

export function getRightSidebarHost(): RightSidebarHost | null {
    return _host;
}
