import { MediaAPI } from '@zealot/api/src/media';
import type { AttributeKind } from '@zealot/domain/src/attribute';
import type { AttributeValueInputBinding, AttributeValueInputOptions } from './attribute_value_input';

const mediaApi = new MediaAPI('/api');

/** Folder, inside the media store, that attribute uploads are written to. */
const UPLOAD_FOLDER = 'attributes';

/** Types handled by {@link createSemanticInput}; all store plain text. */
export const SEMANTIC_BASE_TYPES = ['email', 'telephone', 'url', 'media', 'place'] as const;
export type SemanticBaseType = (typeof SEMANTIC_BASE_TYPES)[number];

export function isSemanticBaseType(baseType: string | undefined): baseType is SemanticBaseType {
    return SEMANTIC_BASE_TYPES.includes(baseType as SemanticBaseType);
}

/**
 * The href a stored value should open, or null when it is not safe or not meaningful to link.
 * Only http(s) URLs, media-store paths, `mailto:`/`tel:` forms of email and telephone values, and a
 * map search for places are produced, so a stored `javascript:` value can never become a link.
 */
export function semanticHref(baseType: SemanticBaseType, value: unknown): string | null {
    if (typeof value !== 'string' || value.trim() === '') {
        return null;
    }
    const text = value.trim();

    switch (baseType) {
        case 'email':
            return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(text) ? `mailto:${text}` : null;
        case 'telephone': {
            const dialable = text.replace(/[^\d+]/g, '');
            return /\d{7,}/.test(dialable) ? `tel:${dialable}` : null;
        }
        case 'url':
        case 'media':
            if (/^https?:\/\/\S+$/i.test(text) || /^\/(api\/)?media\/\S+$/.test(text)) {
                return text;
            }
            return null;
        case 'place': {
            const coordinates = text.match(/^(-?\d+(?:\.\d+)?)\s*,\s*(-?\d+(?:\.\d+)?)$/);
            if (coordinates) {
                return `https://www.openstreetmap.org/?mlat=${coordinates[1]}&mlon=${coordinates[2]}#map=15/${coordinates[1]}/${coordinates[2]}`;
            }
            return `https://www.openstreetmap.org/search?query=${encodeURIComponent(text)}`;
        }
    }
}

const INPUT_TYPES: Record<SemanticBaseType, string> = {
    email: 'email',
    telephone: 'tel',
    url: 'url',
    media: 'text',
    place: 'text',
};

const PLACEHOLDERS: Record<SemanticBaseType, string> = {
    email: 'name@example.com',
    telephone: '+1 555 123 4567',
    url: 'https://',
    media: 'https://… or upload a file',
    place: 'Address, or lat,lon',
};

const LINK_LABELS: Record<SemanticBaseType, string> = {
    email: 'Write email',
    telephone: 'Call',
    url: 'Open link',
    media: 'Open file',
    place: 'Show on map',
};

/**
 * Text input for an email, telephone, url, media or place attribute. A link beside the field stays
 * live while the value is edited, and media values gain an upload button that replaces the value.
 */
export function createSemanticInput(
    baseType: SemanticBaseType,
    options: AttributeValueInputOptions,
): AttributeValueInputBinding {
    const wrapper = document.createElement('span');
    wrapper.className = 'semantic-attribute-input';
    wrapper.style.display = 'inline-flex';
    wrapper.style.gap = '0.5em';
    wrapper.style.alignItems = 'center';

    const input = document.createElement('input');
    input.type = INPUT_TYPES[baseType];
    input.placeholder = PLACEHOLDERS[baseType];
    input.value = options.value != null ? String(options.value) : '';

    const link = document.createElement('a');
    link.className = 'semantic-attribute-link';
    link.textContent = '↗';
    link.rel = 'noopener noreferrer';
    if (baseType === 'url' || baseType === 'media' || baseType === 'place') {
        link.target = '_blank';
    }

    const readValue = (): string | null => {
        const value = input.value.trim();
        return value === '' ? null : value;
    };

    const refreshLink = (): void => {
        const href = semanticHref(baseType, input.value);
        if (href) {
            link.href = href;
            link.title = LINK_LABELS[baseType];
            link.style.display = '';
        } else {
            link.removeAttribute('href');
            link.style.display = 'none';
        }
    };

    const commit = (): void => {
        if (options.onCommit) {
            void Promise.resolve(options.onCommit(readValue()));
        }
    };

    input.addEventListener('input', () => {
        refreshLink();
        options.onValueChange?.(readValue());
    });
    input.addEventListener('blur', commit);
    input.addEventListener('keydown', (event) => {
        if (event.key === 'Enter') {
            event.preventDefault();
            input.blur();
        }
    });

    wrapper.append(input, link);

    if (baseType === 'media') {
        wrapper.appendChild(buildUploadButton(input, () => {
            refreshLink();
            options.onValueChange?.(readValue());
            commit();
        }));
    }

    refreshLink();

    return {
        element: wrapper,
        focus: () => input.focus(),
        getValue: readValue,
    };
}

function buildUploadButton(input: HTMLInputElement, onUploaded: () => void): HTMLElement {
    const picker = document.createElement('input');
    picker.type = 'file';
    picker.hidden = true;

    const button = document.createElement('button');
    button.type = 'button';
    button.textContent = 'Upload';
    button.addEventListener('click', () => picker.click());

    picker.addEventListener('change', () => {
        const file = picker.files?.[0];
        if (!file) {
            return;
        }
        button.disabled = true;
        button.textContent = 'Uploading…';
        const name = `${Date.now()}-${file.name.replace(/[^\w.\-]+/g, '_')}`;
        const upload = new File([file], name, { type: file.type });
        void mediaApi.UploadFolder(upload, UPLOAD_FOLDER)
            .then(() => {
                input.value = `/api/media/${UPLOAD_FOLDER}/${name}`;
                onUploaded();
            })
            .catch(() => {
                button.textContent = 'Upload failed';
            })
            .finally(() => {
                button.disabled = false;
                if (button.textContent === 'Uploading…') {
                    button.textContent = 'Upload';
                }
                picker.value = '';
            });
    });

    const holder = document.createElement('span');
    holder.append(button, picker);
    return holder;
}

/**
 * Checkbox group for a list-of-dropdown attribute (multi-select). Values come from the kind's
 * configured `values`; stored values no longer in that list stay checked so they can be removed.
 */
export function createMultiSelectInput(
    kind: AttributeKind,
    options: AttributeValueInputOptions,
): AttributeValueInputBinding {
    const selected = new Set(Array.isArray(options.value) ? options.value.map(String) : []);
    const choices = [...(kind.Config.values ?? [])];
    selected.forEach((value) => {
        if (!choices.includes(value)) {
            choices.push(value);
        }
    });

    const group = document.createElement('div');
    group.className = 'multi-select-input';
    group.style.display = 'flex';
    group.style.flexWrap = 'wrap';
    group.style.gap = '0.25em 1em';

    const boxes: HTMLInputElement[] = [];
    const readValue = (): string[] => boxes.filter((box) => box.checked).map((box) => box.value);

    choices.forEach((choice) => {
        const label = document.createElement('label');
        const box = document.createElement('input');
        box.type = 'checkbox';
        box.value = choice;
        box.checked = selected.has(choice);
        box.addEventListener('change', () => {
            const next = readValue();
            options.onValueChange?.(next);
            if (options.onCommit) {
                void Promise.resolve(options.onCommit(next));
            }
        });
        boxes.push(box);
        label.append(box, ` ${choice}`);
        group.appendChild(label);
    });

    return {
        element: group,
        focus: () => boxes[0]?.focus(),
        getValue: readValue,
    };
}
