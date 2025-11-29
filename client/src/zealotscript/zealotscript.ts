const token_lex = [
    // Larger tokens first, then smaller
    {
        name: "CodeBlock",
        beginning: '```',
        end: '```'
    },
    {
        name: "Command",
        beginning: "!!!",
        end: "!!!"
    },
    {
        name: ""
    }
]

class ZSNode {
    public Kind: string;
    public Children: ZSNode[] = [];
    public TextContent: string;

    constructor(kind: string) {
        this.Kind = kind;
        this.TextContent = "";
    }

    compile(code: string) {
        if (this.Kind == 'CodeBlock') {
            this.TextContent = code;
            return;
        }

        this.Children = Compile(code);
    }

    render() {

    }
}

export function Compile(input: string): ZSNode[] {
    let nodes = new Array<ZSNode>();

    let beginning = 0;
    let current_mode = 'Text';
    let next_mode = 'Text';

    // Walk through the text, parsing. If we detect the end, then make a node.
    for (let i = 0; i < input.length; i++) {
        

        if (scan_for(input, i, "```")) {
            next_mode = "Code";
        }
        else if (scan_for(input, i, "!!!")) {
            next_mode = "Command";
        }
        else if (scan_for(input, i, ""))
    }

    return nodes;
}

function scan_for(input: string, index: number, token: string): boolean {
    if (input.length < index + token.length) {
        if (input.slice(index, token.length) == token) {
            return true;
        }
        return false;
    }
    return false;
}