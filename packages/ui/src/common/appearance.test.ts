import { beforeEach, describe, expect, it } from 'vitest';
import { applyInterfaceSize } from './appearance';

describe('applyInterfaceSize', () => {
    let root: HTMLElement;
    beforeEach(() => { root = document.createElement('div'); });

    it('zooms the root for each non-default size', () => {
        applyInterfaceSize('small', root);
        expect(root.style.zoom).toBe('0.9');
        applyInterfaceSize('large', root);
        expect(root.style.zoom).toBe('1.12');
        applyInterfaceSize('larger', root);
        expect(root.style.zoom).toBe('1.25');
    });

    it('clears the zoom at the default size', () => {
        applyInterfaceSize('large', root);
        applyInterfaceSize('default', root);
        expect(root.style.zoom).toBe('');
    });

    it('ignores an unknown saved value', () => {
        applyInterfaceSize('huge' as never, root);
        expect(root.style.zoom).toBe('');
    });
});
