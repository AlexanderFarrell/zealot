import type { IconNode } from 'lucide';
import { lookupEmoji } from './emoji_map';
import {
	GENERATED_ICON_CATALOG,
	GENERATED_ICON_SHORTCODES,
	type IconCatalogEntry as GeneratedIconCatalogEntry,
} from './generated/icon_catalog';

type IconCatalogEntry = GeneratedIconCatalogEntry;

type SimpleIconData = {
	svg: string;
	hex: string;
};

type SimpleIconsModule = typeof import('simple-icons');
type LucideModule = typeof import('lucide');
type MdiModule = typeof import('@mdi/js');

const ICON_SHORTCODE_RE = /^:([a-z0-9_+\-]+):$/;

const LEGACY_ICON_CATALOG: Record<string, IconCatalogEntry> = {
	alert: { source: 'lucide', exportName: 'TriangleAlert' },
	book: { source: 'lucide', exportName: 'BookOpen' },
	chart: { source: 'lucide', exportName: 'ChartLine' },
	close: { source: 'lucide', exportName: 'X' },
	cpp: { source: 'simple', exportName: 'siCplusplus' },
	edit: { source: 'lucide', exportName: 'Pencil' },
	flask: { source: 'lucide', exportName: 'FlaskConical' },
	grid: { source: 'lucide', exportName: 'Grid3x3' },
	html: { source: 'simple', exportName: 'siHtml5' },
	java: { source: 'simple', exportName: 'siOpenjdk' },
	lightning: { source: 'mdi', exportName: 'mdiWeatherLightning' },
	nodejs: { source: 'simple', exportName: 'siNodedotjs' },
	snow: { source: 'mdi', exportName: 'mdiWeatherSnowy' },
	trash: { source: 'lucide', exportName: 'Trash2' },
	vue: { source: 'simple', exportName: 'siVuedotjs' },
};

const ICON_CATALOG: Record<string, IconCatalogEntry> = {
	...GENERATED_ICON_CATALOG,
	...LEGACY_ICON_CATALOG,
};

export const ICON_SHORTCODES: string[] = Array.from(new Set([
	...GENERATED_ICON_SHORTCODES,
	...Object.keys(LEGACY_ICON_CATALOG),
])).sort((left, right) => left.localeCompare(right));

const stripTitle = (svg: string): string => svg.replace(/<title>[^<]*<\/title>/g, '');

const normalizeSvgRoot = (
	svg: string,
	attrs: Record<string, string> = {},
): string => svg.replace(/<svg\b([^>]*)>/, (_match, rawAttrs: string) => {
	const cleanedAttrs = rawAttrs
		.replace(/\s(?:aria-hidden|class|data-icon-source|data-native-color|fill|focusable|height|role|style|width)="[^"]*"/g, '')
		.trim();
	const extraAttrs = Object.entries(attrs)
		.map(([key, value]) => `${key}="${value}"`)
		.join(' ');
	return `<svg${cleanedAttrs ? ` ${cleanedAttrs}` : ''}${extraAttrs ? ` ${extraAttrs}` : ''}>`;
});

const simpleIconToSvg = (icon: SimpleIconData): string => normalizeSvgRoot(stripTitle(icon.svg), {
	'aria-hidden': 'true',
	'data-icon-source': 'simple',
	'data-native-color': 'true',
	fill: `#${icon.hex}`,
	focusable: 'false',
});

const lucideToSvg = (data: IconNode): string => {
	const inner = data.map(([tag, attrs]) => {
		const attrStr = Object.entries(attrs as Record<string, string | number>)
			.map(([key, value]) => `${key}="${value}"`)
			.join(' ');
		return `<${tag} ${attrStr}/>`;
	}).join('');
	return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false" data-icon-source="lucide">${inner}</svg>`;
};

const mdiToSvg = (path: string): string =>
	`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true" focusable="false" data-icon-source="mdi"><path d="${path}"/></svg>`;

let simpleIconsPromise: Promise<SimpleIconsModule> | null = null;
let lucidePromise: Promise<LucideModule> | null = null;
let mdiPromise: Promise<MdiModule> | null = null;

const iconSvgCache = new Map<string, string | null>();
const iconSvgLoadPromises = new Map<string, Promise<string | null>>();
const elementRenderVersions = new WeakMap<HTMLElement, number>();

const loadSimpleIconsModule = (): Promise<SimpleIconsModule> => {
	simpleIconsPromise ??= import('simple-icons');
	return simpleIconsPromise;
};

const loadLucideModule = (): Promise<LucideModule> => {
	lucidePromise ??= import('lucide');
	return lucidePromise;
};

const loadMdiModule = (): Promise<MdiModule> => {
	mdiPromise ??= import('@mdi/js');
	return mdiPromise;
};

const svgFromMarkup = (markup: string): SVGSVGElement | null => {
	if (typeof document === 'undefined') return null;
	const template = document.createElement('template');
	template.innerHTML = markup.trim();
	const element = template.content.firstElementChild;
	return element?.tagName.toLowerCase() === 'svg' ? (element as SVGSVGElement) : null;
};

const setFallbackText = (element: HTMLElement, name: string): void => {
	element.replaceChildren();
	element.textContent = `:${name}:`;
};

const setSvgMarkup = (element: HTMLElement, name: string, markup: string): void => {
	const svg = svgFromMarkup(markup);
	if (!svg) {
		setFallbackText(element, name);
		return;
	}
	element.replaceChildren(svg);
};

const getCachedIconSvg = (name: string): string | null | undefined => iconSvgCache.get(name);

const loadSvgFromEntry = async (entry: IconCatalogEntry): Promise<string | null> => {
	if (entry.source === 'simple') {
		const simpleIcons = await loadSimpleIconsModule();
		const icon = (simpleIcons as Record<string, unknown>)[entry.exportName];
		if (
			!icon ||
			typeof icon !== 'object' ||
			typeof (icon as SimpleIconData).svg !== 'string' ||
			typeof (icon as SimpleIconData).hex !== 'string'
		) {
			return null;
		}
		return simpleIconToSvg(icon as SimpleIconData);
	}

	if (entry.source === 'lucide') {
		const lucide = await loadLucideModule();
		const iconNode = (lucide as Record<string, unknown>)[entry.exportName];
		return Array.isArray(iconNode) ? lucideToSvg(iconNode as IconNode) : null;
	}

	const mdi = await loadMdiModule();
	const path = (mdi as Record<string, unknown>)[entry.exportName];
	return typeof path === 'string' ? mdiToSvg(path) : null;
};

export const hasIcon = (name: string): boolean => Object.prototype.hasOwnProperty.call(ICON_CATALOG, name);

export const loadIconSvg = (name: string): Promise<string | null> => {
	const entry = ICON_CATALOG[name];
	if (!entry) {
		return Promise.resolve(null);
	}

	const cached = getCachedIconSvg(name);
	if (cached !== undefined) {
		return Promise.resolve(cached);
	}

	const pending = iconSvgLoadPromises.get(name);
	if (pending) {
		return pending;
	}

	const loadPromise = loadSvgFromEntry(entry)
		.then((svg) => {
			iconSvgCache.set(name, svg);
			iconSvgLoadPromises.delete(name);
			return svg;
		})
		.catch(() => {
			iconSvgCache.set(name, null);
			iconSvgLoadPromises.delete(name);
			return null;
		});

	iconSvgLoadPromises.set(name, loadPromise);
	return loadPromise;
};

export const parseIconShortcode = (raw: string): string | null => {
	const match = ICON_SHORTCODE_RE.exec(raw.trim());
	return match?.[1] ?? null;
};

export const setIconRefElement = (
	element: HTMLElement,
	name: string,
	options: { className?: string; title?: string } = {},
): void => {
	element.className = options.className ?? 'zealot-icon';
	element.setAttribute('data-icon-ref', name);
	element.setAttribute('title', options.title ?? name);

	const renderVersion = (elementRenderVersions.get(element) ?? 0) + 1;
	elementRenderVersions.set(element, renderVersion);

	const cachedSvg = getCachedIconSvg(name);
	if (cachedSvg !== undefined) {
		if (cachedSvg) {
			setSvgMarkup(element, name, cachedSvg);
		} else {
			setFallbackText(element, name);
		}
		return;
	}

	setFallbackText(element, name);
	if (!hasIcon(name)) {
		return;
	}

	void loadIconSvg(name).then((svg) => {
		if (!svg) return;
		if (elementRenderVersions.get(element) !== renderVersion) return;
		if (element.getAttribute('data-icon-ref') !== name) return;
		setSvgMarkup(element, name, svg);
	});
};

export const createIconRefElement = (
	name: string,
	options: { className?: string; title?: string } = {},
): HTMLElement => {
	const element = document.createElement('span');
	setIconRefElement(element, name, options);
	return element;
};

export const createRawIconElement = (
	raw: string,
	options: { className?: string; title?: string } = {},
): HTMLElement | null => {
	const name = parseIconShortcode(raw);
	if (!name || lookupEmoji(name) || !hasIcon(name)) return null;
	return createIconRefElement(name, options);
};
