import { AppSettings, type InterfaceSize } from '@websoil/engine';

/** Whole-app scale for each Interface size. The stylesheets use pixels, so the root is zoomed. */
export const INTERFACE_ZOOM: Record<InterfaceSize, number> = {
    small: 0.9,
    default: 1,
    large: 1.12,
    larger: 1.25,
};

export function applyInterfaceSize(size: InterfaceSize, root: HTMLElement = document.documentElement): void {
    const zoom = INTERFACE_ZOOM[size] ?? 1;
    root.style.zoom = zoom === 1 ? '' : String(zoom);
}

/** Applies the saved interface size now and whenever the setting changes (including after login). */
export function initAppearance(): void {
    applyInterfaceSize(AppSettings.get().interface_size);
    AppSettings.subscribe((settings) => applyInterfaceSize(settings.interface_size));
}
