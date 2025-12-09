import API from "../../api/api";
import type { Item } from "../../api/item";
import BaseElement from "../../components/common/base_element";

interface ContentBlock {
    content: string,
    kind: string,
    children: []
}

// Main
const Root = 0;

// Blocks
const Text = 1;
const Code = 2;
const Command = 3;

// Specific Text Kinds
const Heading = 4;
const Admonition = 5;
const BulletPoint = 6;
const NumberPoint = 7;

function IsAhead(str: string, start: number, term: string): boolean {
    if (str.length < start + term.length) {
        return false;
    }
    return str.substring(start, term.length + start) == term;
}

class ZealotBlock {
    public Content: string;
    // public Raw: string;
    public Kind: number;
    public Children: ZealotBlock[] = []

    constructor(raw: string = "", kind: number = Root) {
        this.Content = raw;
        this.Kind = kind;
        this.compile(raw);
    }

    public compile(raw: string) {
        this.Children = [];

        // In code types, the whole thing is just code. Just return.
        if (this.Kind == Code) {
            this.Content = raw;
            return;
        } else if (this.Kind == Root) {
            // Find blocks. This changes modes.
            let mode = Text;
            let start = 0;
            let end = 0;

            let make_new_block = (offset: number = 1) => {
                this.Children.push(new ZealotBlock(raw.substring(start, end), mode));
                start += offset;
                end += offset - 1;
            }

            while (end < raw.length) {
                if (IsAhead(raw, end, '\n')) {
                    if (mode == Text) {
                        make_new_block(1);
                    }
                } else if (IsAhead(raw, end, '```')) {
                    // Start or end code block
                    if (mode == Code) {
                        make_new_block(3);
                        // End the code block
                        mode = Text;
                    } else {
                        make_new_block(3);
                        // Start the code block
                        mode = Code;
                    }
                } else if (IsAhead(raw, end, '{{{')) {
                    // Trim the remaining
                    // Start a new block
                    make_new_block(3);
                    mode = Command;
                } else if (IsAhead(raw, end, '}}}')) {
                    // Make a new command block
                    // Then go back to text
                    make_new_block(3);
                    mode = Text;
                } 
                end++;
            }
            
            // Bring Bullet Point and Admonitions together after we parse
            // They already would have figured out what they where
        } else if (this.Kind = Text) {
            // We are a single line of text, figure out if more.
            
            if (this.Content.charAt(0) == "#") {
                this.Kind = Heading;
            } else if (/^\s*[-*] $/.test(this.Content)) {
                // Bullet point
                this.Kind = BulletPoint;
            } else if (/^\s*\d+\. $/.test(this.Content)) {
                // Numbered list item
                this.Kind = NumberPoint;
            } else if (/^\s*[>] $/.test(this.Content)) {
                this.Kind = Admonition;
            }


            // Then parse any inline stuff!

            let start = 0;
            let end = 0;

            while (end < this.Content.length) {
                // Look for more blocks
                if (IsAhead(this.Content, end, "**")) {
                    // Bold
                } else if (IsAhead(this.Content, end, "~~")) {
                    // Strikethrough
                }

                end++;
            }
        }

        




        // if (this.Kind == "root") {
        //     this.parse_large_blocks()
        //     // Find major blocks

        // } else {
        //     this.Content = raw;
        // }
    }



    private parse_line_blocks() {
        let lines = raw.split('\n');
        this.Content = "";
        this.Blocks = lines.map(l => {
            return new ZealotBlock(l, 'p');
        })
    }

    public decompile(): string {
        let code = '';
        this.Blocks.forEach(block => {
            code += block.Content;
        })
        return code;
    }

    public insert(child: ZealotBlock, index: number = -1) {
        if (index == -1) {
            this.Blocks.push(child)
        } else {
            this.Blocks.splice(index, 0, child);
        }
    }
}


class ContentView extends BaseElement<Item> {
    private dsl!: ZealotBlock;
    private elements: HTMLElement[] = [];
    private time_since_save: number = 0;
    private save_loop: any;

    render() {
        let item = this.data!;

        // Convert to DSL
        this.dsl = new ZealotBlock(item.content, "root")

        // Then render
        this.refresh()

        this.save_loop = setInterval(() => {
            if (this.time_since_save == 1) {
                this.save();
            }

            if (this.time_since_save > 0) {
                this.time_since_save--;
            }
        }, 1000);
    }

    refresh() {
        let add_block = (block: ContentBlock, index: number): HTMLElement => {
            let element = document.createElement(block.kind);
            element.innerText = block.content;
            element.addEventListener('keydown', (e: KeyboardEvent) => {
                this.time_since_save = 5;
                if (e.key == "Enter") {
                    e.preventDefault();
                    add_block({
                        content: "",
                        kind: 'p',
                        children: []
                    }, index+1).focus();
                }
                if (e.key == "ArrowUp") {

                }
                // this.dsl[index] = element.textContent;
            })
            element.contentEditable = "true";
            this.elements.splice(index, 0, element);
            this.appendChild(element);
            return element;
        } 
        this.dsl.Blocks.forEach((block, index) => {
            add_block(block, index)
        })
    }

    disconnectCallback() {
        this.save();
    }

    save() {
        let content = this.dsl.map(d => {return d.content}).join('\n');
        this.data!.content = content;
        API.item.update(this.data!.item_id, {content: content})
    }
}

customElements.define('content-view', ContentView);

export default ContentView;