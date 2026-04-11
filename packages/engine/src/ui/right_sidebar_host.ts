export interface RightSidebarHost {
    setContent(el: HTMLElement | null): void;
}

let _host: RightSidebarHost | null = null;

export function setRightSidebarHost(host: RightSidebarHost): void {
    _host = host;
}

export function getRightSidebarHost(): RightSidebarHost | null {
    return _host;
}
