import runner from './commands';

export type ToolView = 'search' | 'nav_tree' | 'calendar' | 'random';

export interface ToolShowOptions {
    focus?: boolean;
}

export interface ToolHost {
    show(view: ToolView, options?: ToolShowOptions): void;
    toggle(view: ToolView, options?: ToolShowOptions): void;
}

export const ToolCommands = {
    searchItems: 'Search Items',
    openNavTree: 'Open Nav Sidebar',
    openCalendar: 'Open Calendar',
    openRandom: 'Open Random Items',
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
        getToolHost().toggle('search', { focus: true });
    });
    runner.register(ToolCommands.openNavTree, [], () => getToolHost().toggle('nav_tree'));
    runner.register(ToolCommands.openCalendar, [], () => getToolHost().toggle('calendar'));
    runner.register(ToolCommands.openRandom, [], () => getToolHost().toggle('random'));
}
