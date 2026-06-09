import { clear_hotkeys, register_hotkey, type Hotkey } from "./hotkeys";

export type UICommand = {
    name: string,
    hotkeys: Hotkey[],
    func: () => void
}

class CommandRunner {
    public Commands: Map<string, UICommand> = new Map();

    register(name: string, hotkeys: Hotkey[], on: () => void) {
        this.Commands.set(name, {
            name: name,
            hotkeys: hotkeys,
            func: on
        });
        hotkeys.forEach(k => {
            k.func = () => {this.run(name)};
            register_hotkey(k)
        })
    }

    run(name: string) {
        if (!this.Commands.has(name)) {
            console.error("No such command: " + name);
            return;
        }

        let c = this.Commands.get(name)!;
        c.func();
    }

    search_commands(term: string): Array<UICommand> {
        const lower = term.toLowerCase();
        const exact: UICommand[] = [];
        const partial: UICommand[] = [];
        for (const [key, value] of this.Commands.entries()) {
            const kl = key.toLowerCase();
            if (kl === lower) exact.push(value);
            else if (kl.includes(lower)) partial.push(value);
        }
        partial.sort((a, b) => a.name.localeCompare(b.name));
        return [...exact, ...partial];
    }

    clear() {
        this.Commands = new Map();
        clear_hotkeys();
    }
}

export let runner = new CommandRunner();

export default runner;
