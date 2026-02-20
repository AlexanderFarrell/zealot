import { register_hotkey, type Hotkey } from "./hotkeys";

export type UICommand = {
    name: string,
    hotkeys: Hotkey[],
    func: Function
}

class CommandRunner {
    public Commands: Map<string, UICommand> = new Map();

    register(name: string, hotkeys: Hotkey[], on: Function) {
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
        let retVal: UICommand[] = [];
        for (const [key, value] of this.Commands.entries()) {
            if (key.toLowerCase().includes(term.toLowerCase())) {
                retVal.push(value)
            }
        }
        retVal.sort((a, b) => {
            return a.name.localeCompare(b.name);
        })

        return retVal;
    }

    clear() {
        this.Commands = new Map();
    }
}

export let runner = new CommandRunner();

export default runner;