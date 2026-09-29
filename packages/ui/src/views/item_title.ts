import type { Item } from '@zealot/domain/src/item';
import { lookupEmoji } from '../zealotscript/emoji_map';
import { createRawIconElement } from '../zealotscript/icon_registry';

export type ItemTitleLike = Pick<Item, 'Title' | 'RawIcon'>;

type ItemTitleOptions = {
	className?: string;
	iconClassName?: string;
	textClassName?: string;
};

const SHORTCODE_RE = /^:([a-z0-9_+\-]+):$/;

const joinClasses = (...classes: Array<string | undefined>): string =>
	classes.filter((value) => value && value.trim().length > 0).join(' ');

const createTextIconElement = (
	text: string,
	options: { className?: string; title?: string } = {},
): HTMLElement | null => {
	const content = text.trim();
	if (!content) return null;
	const element = document.createElement('span');
	element.className = joinClasses('item-title-inline-icon', 'item-title-inline-icon--text', options.className);
	element.textContent = content;
	if (options.title) {
		element.setAttribute('title', options.title);
	}
	return element;
};

export const createItemTitleIconElement = (
	item: ItemTitleLike,
	options: { className?: string; title?: string } = {},
): HTMLElement | null => {
	const title = options.title ?? item.Title;
	const textIconOptions = options.className
		? { className: options.className, title }
		: { title };
	const svgIcon = createRawIconElement(item.RawIcon, {
		className: joinClasses('zealot-icon', 'item-title-inline-icon', options.className),
		title,
	});
	if (svgIcon) {
		return svgIcon;
	}

	const raw = item.RawIcon.trim();
	if (!raw) {
		return null;
	}

	const shortcode = SHORTCODE_RE.exec(raw)?.[1];
	if (shortcode) {
		const emoji = lookupEmoji(shortcode);
		if (emoji) {
			return createTextIconElement(emoji, textIconOptions);
		}
	}

	return createTextIconElement(raw, textIconOptions);
};

export const createItemTitleElement = (
	item: ItemTitleLike,
	options: ItemTitleOptions = {},
): HTMLSpanElement => {
	const element = document.createElement('span');
	element.className = joinClasses('item-title-inline', options.className);

	const icon = createItemTitleIconElement(
		item,
		options.iconClassName ? { className: options.iconClassName } : {},
	);
	if (icon) {
		element.appendChild(icon);
	}

	const text = document.createElement('span');
	text.className = joinClasses('item-title-text', options.textClassName);
	text.textContent = item.Title;
	element.appendChild(text);

	return element;
};

export const renderItemTitle = (
	container: HTMLElement,
	item: ItemTitleLike,
	options: ItemTitleOptions = {},
): void => {
	container.replaceChildren(createItemTitleElement(item, options));
};
