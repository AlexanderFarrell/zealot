export type DesktopMode = 'remote' | 'local';

let _mode: DesktopMode | null = null;
let _remoteHost: string | null = null;

/** Called once at desktop app init before any UI renders. */
export function setDesktopMode(mode: DesktopMode, remoteHost?: string): void {
    _mode = mode;
    _remoteHost = mode === 'remote' ? (remoteHost ?? null) : null;
}

/** Returns the current desktop mode, or null when running as a plain web app. */
export function getDesktopMode(): DesktopMode | null {
    return _mode;
}

/** Returns the remote server hostname when in remote mode, otherwise null. */
export function getRemoteHost(): string | null {
    return _remoteHost;
}
