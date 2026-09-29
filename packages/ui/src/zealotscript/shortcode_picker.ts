import { EMOJI_MAP, lookupEmoji } from "./emoji_map";
import { ICON_SHORTCODES, setIconRefElement } from "./icon_registry";

export type ShortcodeSuggestion =
	| { kind: "emoji"; shortcode: string; emoji: string }
	| { kind: "icon"; shortcode: string };

export type ShortcodeTrigger = {
	from: number;
	query: string;
};

type ShortcodePickerAnchor = {
	left: number;
	bottom: number;
};

type ShortcodePickerOptions = {
	containsTarget: (target: Node) => boolean;
	onSelect: (suggestion: ShortcodeSuggestion) => void;
};

const SHORTCODE_QUERY_RE = /^[a-z0-9_+\-]*$/;
const SHORTCODE_SUGGESTION_LIMIT = 12;

const compareMatches = (
	query: string,
	left: ShortcodeSuggestion,
	right: ShortcodeSuggestion,
): number => {
	if (left.kind !== right.kind) {
		return left.kind === "emoji" ? -1 : 1;
	}

	const leftIndex = left.shortcode.indexOf(query);
	const rightIndex = right.shortcode.indexOf(query);
	if (leftIndex !== rightIndex) {
		return leftIndex - rightIndex;
	}

	if (left.shortcode.length !== right.shortcode.length) {
		return left.shortcode.length - right.shortcode.length;
	}

	return left.shortcode.localeCompare(right.shortcode);
};

export const detectShortcodeTriggerInText = (text: string): ShortcodeTrigger | null => {
	const lastColon = text.lastIndexOf(":");
	if (lastColon === -1) return null;
	const query = text.slice(lastColon + 1);
	if (query.length === 0) return null;
	if (query.includes(":") || query.includes(" ") || query.includes("\n")) return null;
	if (!SHORTCODE_QUERY_RE.test(query)) return null;
	return { from: lastColon, query };
};

export const getShortcodeSuggestions = (query: string): ShortcodeSuggestion[] => {
	const normalizedQuery = query.toLowerCase();
	const emojiMatches: ShortcodeSuggestion[] = Object.entries(EMOJI_MAP)
		.filter(([shortcode]) => shortcode.includes(normalizedQuery))
		.map(([shortcode, emoji]) => ({ kind: "emoji", shortcode, emoji }));
	const iconMatches: ShortcodeSuggestion[] = ICON_SHORTCODES
		.filter((shortcode) => !lookupEmoji(shortcode) && shortcode.includes(normalizedQuery))
		.map((shortcode) => ({ kind: "icon", shortcode }));

	return [...emojiMatches, ...iconMatches]
		.sort((left, right) => compareMatches(normalizedQuery, left, right))
		.slice(0, SHORTCODE_SUGGESTION_LIMIT);
};

export class ShortcodePicker {
	private pickerEl: HTMLDivElement | null = null;
	private outsideClickListener: ((e: MouseEvent) => void) | null = null;

	constructor(private readonly options: ShortcodePickerOptions) {}

	get isOpen(): boolean {
		return this.pickerEl !== null;
	}

	update(anchor: ShortcodePickerAnchor, query: string): void {
		const matches = getShortcodeSuggestions(query);
		if (matches.length === 0) {
			this.hide();
			return;
		}

		const el = this.ensureElement();
		el.style.position = "fixed";
		el.style.top = `${anchor.bottom + 4}px`;
		el.style.left = `${anchor.left}px`;
		el.replaceChildren();

		for (const match of matches) {
			const btn = document.createElement("button");
			btn.type = "button";
			btn.className = "zealotscript-emoji-option";

			const glyph = document.createElement("span");
			glyph.className = "zealotscript-shortcode-option-glyph";

			const label = document.createElement("span");
			label.className = "zealotscript-shortcode-option-label";
			label.textContent = match.shortcode;

			if (match.kind === "emoji") {
				glyph.textContent = match.emoji;
			} else {
				setIconRefElement(glyph, match.shortcode, {
					className: "zealot-icon zealotscript-shortcode-option-glyph",
					title: match.shortcode,
				});
			}

			btn.addEventListener("mousedown", (event) => {
				event.preventDefault();
				this.hide();
				this.options.onSelect(match);
			});

			btn.append(glyph, label);
			el.appendChild(btn);
		}
	}

	hide(): void {
		if (this.outsideClickListener) {
			document.removeEventListener("mousedown", this.outsideClickListener);
			this.outsideClickListener = null;
		}
		this.pickerEl?.remove();
		this.pickerEl = null;
	}

	private ensureElement(): HTMLDivElement {
		if (this.pickerEl) {
			return this.pickerEl;
		}

		const wrapper = document.createElement("div");
		wrapper.className = "zealotscript-emoji-picker";
		document.body.appendChild(wrapper);
		this.pickerEl = wrapper;

		this.outsideClickListener = (event: MouseEvent) => {
			const target = event.target;
			if (
				this.pickerEl &&
				target instanceof Node &&
				!this.pickerEl.contains(target) &&
				!this.options.containsTarget(target)
			) {
				this.hide();
			}
		};
		document.addEventListener("mousedown", this.outsideClickListener);

		return wrapper;
	}
}
