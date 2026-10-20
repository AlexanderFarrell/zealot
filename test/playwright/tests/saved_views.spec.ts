import type { APIRequestContext } from '@playwright/test';
import { test, expect } from '../fixtures';
import { csrfHeaders } from '../helpers/csrf';
import { uniqueTitle } from '../helpers/unique';
import { APP_SHELL } from '../helpers/selectors';

interface NewItem {
  title: string;
  content?: string;
  types?: string[];
  attributes?: Record<string, unknown>;
}

const create = async (request: APIRequestContext, item: NewItem): Promise<number> => {
  const res = await request.post('/api/item', {
    data: { content: '', ...item },
    headers: await csrfHeaders(request),
  });
  if (!res.ok()) throw new Error(`create ${item.title} failed: ${res.status()} ${await res.text()}`);
  return ((await res.json()) as { item_id: number }).item_id;
};

/** The saved content of an item, read back from the API (the editor and save queue both debounce). */
const storedContent = async (request: APIRequestContext, title: string): Promise<string> => {
  const res = await request.get(`/api/item/title/${encodeURIComponent(title)}`);
  return ((await res.json()) as { content: string }).content;
};

const block = (definition: object): string =>
  `What this shows.\n\n\`\`\`zealot-view\n${JSON.stringify(definition, null, 2)}\n\`\`\`\n`;

/** A parent with a Working and a Complete child, and a Saved View over everything under it. */
const seed = async (request: APIRequestContext) => {
  const parent = await create(request, { title: uniqueTitle('ViewParent') });
  const working = uniqueTitle('ViewWorking');
  const done = uniqueTitle('ViewDone');
  await create(request, { title: working, attributes: { Parent: [parent], Status: 'Working' } });
  await create(request, { title: done, attributes: { Parent: [parent], Status: 'Complete' } });
  const viewTitle = uniqueTitle('ViewUnder');
  await create(request, {
    title: viewTitle,
    types: ['Saved View'],
    content: block({ query: { under: parent, sort: { key: 'title' } } }),
  });
  return { parent, working, done, viewTitle };
};

test.beforeEach(async ({ page }) => {
  await page.goto('/');
  await expect(page.locator(APP_SHELL.webClient)).toBeVisible({ timeout: 10_000 });
});

test('a Saved View page shows its live result, and cards/table sticks after reload', async ({ page, request }) => {
  const { working, done, viewTitle } = await seed(request);
  await page.goto(`/item/${encodeURIComponent(viewTitle)}`);

  const box = page.locator('.zealot-view-block .view-box');
  await expect(box).toBeVisible();
  await expect(box.locator('.view-box__total')).toHaveText('2 items');
  await expect(box.locator('.item-card')).toHaveCount(2);
  await expect(box).toContainText(working);
  await expect(box).toContainText(done);

  await box.getByRole('button', { name: 'Table' }).click();
  await expect(box.locator('item-table-view')).toBeVisible();
  await expect(box.locator('.item-card')).toHaveCount(0);

  // The mode is written into the definition and saved through the normal path.
  await expect.poll(() => storedContent(request, viewTitle), { timeout: 15_000 }).toContain('"display": "table"');
  await page.reload();
  await expect(page.locator('.zealot-view-block .view-box item-table-view')).toBeVisible();
});

test('![[View]] embeds the result in any item, and its table toggle is remembered', async ({ page, request }) => {
  const { viewTitle } = await seed(request);
  const host = uniqueTitle('ViewHost');
  await create(request, { title: host, content: `Intro\n\n![[${viewTitle}]]\n\nAfter` });
  await page.goto(`/item/${encodeURIComponent(host)}`);

  const box = page.locator('.zealot-view-embed .view-box');
  await expect(box).toBeVisible();
  await expect(box.locator('.view-box__total')).toHaveText('2 items');

  await box.getByRole('button', { name: 'Table' }).click();
  await expect(box.locator('item-table-view')).toBeVisible();
  await expect.poll(() => storedContent(request, host), { timeout: 15_000 }).toContain(`![[${viewTitle}|table]]`);

  await page.reload();
  await expect(page.locator('.zealot-view-embed .view-box item-table-view')).toBeVisible();
});

test('an embed of something that is not a Saved View says so', async ({ page, request }) => {
  const plain = uniqueTitle('NotAView');
  await create(request, { title: plain });
  const host = uniqueTitle('ViewHostBad');
  await create(request, { title: host, content: `![[${plain}]]` });
  await page.goto(`/item/${encodeURIComponent(host)}`);
  await expect(page.locator('.zealot-view-embed .view-box__error')).toContainText('Not a Saved View');
});

test('the builder edits a filter, previews it, and Done rewrites the block', async ({ page, request }) => {
  const { working, done, viewTitle } = await seed(request);
  await page.goto(`/item/${encodeURIComponent(viewTitle)}`);

  await page.locator('.zealot-view-block').getByRole('button', { name: 'Edit view' }).click();
  const builder = page.locator('.view-builder');
  await expect(builder).toBeVisible();
  await expect(builder.locator('.view-box .item-card')).toHaveCount(2);

  await builder.getByRole('button', { name: 'Add filter' }).first().click();
  await builder.locator('.view-builder__choice', { hasText: 'Working' }).locator('input').check();

  // The preview reruns the unsaved definition after the debounce.
  await expect(builder.locator('.view-box .view-box__total')).toHaveText('1 item');
  await expect(builder.locator('.view-box')).toContainText(working);
  await expect(builder.locator('.view-box')).not.toContainText(done);

  await builder.getByRole('button', { name: 'Done' }).click();
  await expect(page.locator('.view-builder')).toHaveCount(0);
  await expect(page.locator('.zealot-view-block .view-box__total')).toHaveText('1 item');

  await expect.poll(() => storedContent(request, viewTitle), { timeout: 15_000 }).toContain('"value": "Working"');
  expect(await storedContent(request, viewTitle)).toContain('"key": "Status"');
});

test('New Saved View from the command palette opens in Edit view with a starter block', async ({ page, request }) => {
  await page.keyboard.press('Control+p');
  const palette = page.locator('command-palette-modal');
  await expect(palette).toBeVisible();
  await palette.locator('input').fill('New Saved View');
  await palette.locator('input').press('Enter');

  const title = uniqueTitle('ViewNew');
  await expect(page.locator('add-item-modal')).toBeVisible();
  await expect(page.locator('add-item-modal [name="item_type"]')).toHaveValue('Saved View');
  await page.fill('add-item-modal [name="title"]', title);
  await page.click('add-item-modal [data-role="submit"]');

  await expect(page.locator('.zealot-view-block .view-builder')).toBeVisible();
  await expect(page.locator('.view-builder .view-box')).toBeVisible();
  const content = await storedContent(request, title);
  expect(content).toContain('```zealot-view');
});

test('typing ![[View]] on its own line turns into the embed at once', async ({ page, request }) => {
  const { viewTitle } = await seed(request);
  const host = uniqueTitle('ViewHostTyped');
  await create(request, { title: host, content: '' });
  await page.goto(`/item/${encodeURIComponent(host)}`);

  await page.locator('.ProseMirror').click();
  await page.keyboard.type(`![[${viewTitle}]]`);
  await expect(page.locator('.zealot-view-embed .view-box')).toBeVisible();
  await expect(page.locator('.zealot-view-embed .view-box__total')).toHaveText('2 items');
  await expect.poll(() => storedContent(request, host), { timeout: 15_000 }).toContain(`![[${viewTitle}]]`);
});
