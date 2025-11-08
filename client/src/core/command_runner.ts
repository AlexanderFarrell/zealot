import { register_hotkey, type Hotkey } from "./hotkeys";


class CommandRunner {
    public Commands: Map<string, Function> = new Map();

    register(name: string, hotkeys: Hotkey[], on: Function) {
        this.Commands.set(name, on);
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

        let on = this.Commands.get(name)!;
        on();
    }

    search_commands(term: string): Array<object> {
        let retVal = [];
        for (const [key, value] of this.Commands.entries()) {
            if (key.includes(term)) {
                retVal.push({
                    name: key,
                    func: value
                })
            }
        }
        retVal.sort((a, b) => {
            return a.name.localeCompare(b.name);
        })

        return retVal;
    }
}

let runner = new CommandRunner();

export default runner;