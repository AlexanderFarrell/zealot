import runner from './commands';

export type ToolView = 'search' | 'nav_tree' | 'calendar' | 'item_attributes';

export interface ToolShowOptions {
    focus?: boolean;
}

export interface ToolHost {
    show(view: ToolView, options?: ToolShowOptions): void;
}

export const ToolCommands = {
    searchItems: 'Search Items',
    openNavTree: 'Open Nav Sidebar',
    openCalendar: 'Open Calendar',
} as const;

let _toolHost: ToolHost | null = null;

export function setToolHost(host: ToolHost): void {
    _toolHost = host;
}

export function getToolHost(): ToolHost {
    if (!_toolHost) {
        throw new Error('Tool host not registered');
    }
    return _toolHost;
}

export function registerToolCommands(): void {
    runner.register(ToolCommands.searchItems, [], () => {
        getToolHost().show('search', { focus: true });
    });
    runner.register(ToolCommands.openNavTree, [], () => getToolHost().show('nav_tree'));
    runner.register(ToolCommands.openCalendar, [], () => getToolHost().show('calendar'));
}
