import { test, expect } from '../fixtures';
import { uniqueTitle } from '../helpers/unique';

const EDITOR = 'item-screen .item-content zealotscript-editor';
const NUMBERS = `${EDITOR} .ProseMirror ol.zealot-outline li[data-outline-number]`;

const OUTLINE = ':::outline\n1. Goals\n  1.1 [x] Learn\n  1.2 [ ] Ship\n2. Risks\n:::';

const numbers = async (page: import('@playwright/test').Page): Promise<string[]> =>
  page.locator(NUMBERS).evaluateAll((els) => els.map((el) => el.getAttribute('data-outline-number') ?? ''));

const savedContent = async (request: import('@playwright/test').APIRequestContext, id: number): Promise<string> => {
  const res = await request.get(`/api/item/id/${id}`);
  const body = await res.json() as { content: string };
  return body.content;
};

test('outline renders numbers, renumbers on indent, and saves them in the text', async ({ page, request, createItem }) => {
  const title = uniqueTitle('Outline');
  const id = await createItem(title, OUTLINE);
  await page.goto(`/item/${encodeURIComponent(title)}`);
  await expect(page.locator(EDITOR)).toBeVisible();

  await expect.poll(() => numbers(page)).toEqual(['1.', '1.1', '1.2', '2.']);
  await expect(page.locator(`${EDITOR} .ProseMirror li[data-outline-number="1.1"] input[type=checkbox]`)).toBeChecked();

  // Indent "Risks" under "Goals": it becomes 1.3 and the top level has one item.
  await page.locator(`${EDITOR} .ProseMirror`).getByText('Risks').click();
  // ProseMirror syncs the DOM selection a few ms after the click; Tab sooner lands at the start.
  await page.waitForTimeout(150);
  await page.keyboard.press('Tab');
  await expect.poll(() => numbers(page)).toEqual(['1.', '1.1', '1.2', '1.3']);

  // The saved text carries the computed number, so a raw reader can cite "1.3".
  await expect.poll(async () => savedContent(request, id), { timeout: 8_000 }).toContain('  1.3 Risks');

  await page.reload();
  await expect.poll(() => numbers(page)).toEqual(['1.', '1.1', '1.2', '1.3']);
  await page.screenshot({ path: 'test-results/outline-after-reload.png' });
});

test('toolbar converts a bullet list to an outline and back, keeping checkboxes', async ({ page, request, createItem }) => {
  const title = uniqueTitle('OutlineConvert');
  const id = await createItem(title, '- [x] a\n\t- [ ] b\n- c');
  await page.goto(`/item/${encodeURIComponent(title)}`);
  await expect(page.locator(EDITOR)).toBeVisible();

  await page.locator(`${EDITOR} .ProseMirror`).getByText('b', { exact: true }).click();
  await page.waitForTimeout(150);
  await page.locator(`${EDITOR} button`, { hasText: '1.1 Outline' }).click();
  await expect.poll(() => numbers(page)).toEqual(['1.', '1.1', '2.']);
  await expect.poll(async () => savedContent(request, id), { timeout: 8_000 })
    .toBe(':::outline\n1. [x] a\n  1.1 [ ] b\n2. c\n:::');

  await page.locator(`${EDITOR} button`, { hasText: '• List' }).click();
  await expect.poll(async () => savedContent(request, id), { timeout: 8_000 })
    .toBe('- [x] a\n\t- [ ] b\n- c');
});
