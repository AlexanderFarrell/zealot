import { test, expect } from '../fixtures';
import { uniqueTitle } from '../helpers/unique';
import { ITEM, SEARCH, CONFIRM } from '../helpers/selectors';

test('archive preserves history, hides search, and restores through the UI', async ({ page, request, createItem }) => {
  const title = uniqueTitle('Archive');
  const id = await createItem(title, '# Preserved history\nAll content stays.');
  await page.goto(`/item/${encodeURIComponent(title)}`);
  await expect(page.locator(ITEM.title)).toHaveText(title);
  const before = await (await request.get(`/api/item/id/${id}`)).json();
  await page.getByRole('button', { name: 'Archive Item', exact: true }).click();
  await page.locator(CONFIRM.confirmButton).click();
  await expect(page.locator('.item-archive-banner')).toBeVisible();
  const archived = await (await request.get(`/api/item/id/${id}`)).json();
  expect(archived.archived).toBe(true);
  expect(archived.content).toBe(before.content);
  expect(archived.types).toEqual(before.types);
  expect(archived.links).toEqual(before.links);
  expect(archived.attributes).toEqual({ ...before.attributes, Archived: true });
  await page.keyboard.press('Control+o');
  await page.locator(SEARCH.input).fill(title);
  await expect(page.locator(SEARCH.status)).toHaveText('No results.');
  await page.getByRole('checkbox', { name: 'Include archived' }).check();
  await expect(page.locator(SEARCH.resultRow)).toContainText(title);
  await page.locator(SEARCH.resultRow).click();
  await expect(page.locator('.item-archive-banner')).toBeVisible();
  await page.getByRole('button', { name: 'Unarchive Item', exact: true }).click();
  await page.locator(CONFIRM.confirmButton).click();
  await expect(page.locator('.item-archive-banner')).toHaveCount(0);
  await page.keyboard.press('Control+o');
  await page.locator(SEARCH.input).fill(title);
  await expect(page.locator(SEARCH.resultRow)).toContainText(title);
});

test('subtree archive and unarchive retain Parent links and wikilink resolution', async ({ page, request, createItem }) => {
  const title = uniqueTitle('ArchiveTree');
  const root = await createItem(title);
  const childTitle = uniqueTitle('ArchiveChild');
  const child = await createItem(childTitle, `[[${title}]]`);
  const csrf = (await request.storageState()).cookies.find(c => c.name === 'csfr_')?.value;
  const headers = csrf ? { 'X-Csrf-Token': csrf } : undefined;
  expect((await request.patch(`/api/item/${child}/attr`, { data: { Parent: [root] }, headers })).ok()).toBe(true);
  await page.goto(`/item/${encodeURIComponent(title)}`);
  await page.getByRole('button', { name: 'Archive Subtree', exact: true }).click();
  await expect(page.locator('confirm-dialog')).toContainText('all descendants');
  await page.locator(CONFIRM.confirmButton).click();
  await expect(page.locator('.item-archive-banner')).toBeVisible();
  const archivedChild = await (await request.get(`/api/item/id/${child}`)).json();
  expect(archivedChild.archived).toBe(true);
  expect(archivedChild.attributes.Parent).toEqual([root]);
  expect(archivedChild.content).toBe(`[[${title}]]`);
  // Direct title reads, also used to resolve wikilinks, still succeed.
  expect((await request.get(`/api/item/title/${encodeURIComponent(childTitle)}`)).ok()).toBe(true);
  await page.getByRole('button', { name: 'Unarchive Subtree', exact: true }).click();
  await page.locator(CONFIRM.confirmButton).click();
  await expect(page.locator('.item-archive-banner')).toHaveCount(0);
  expect((await (await request.get(`/api/item/id/${child}`)).json()).archived).toBe(false);
});

test('cancelling an archive leaves the item visible', async ({ page, request, createItem }) => {
  const title = uniqueTitle('KeepActive');
  const id = await createItem(title);
  await page.goto(`/item/${encodeURIComponent(title)}`);
  await page.getByRole('button', { name: 'Archive Item', exact: true }).click();
  await page.locator('confirm-dialog').getByRole('button', { name: 'Cancel' }).click();
  expect((await (await request.get(`/api/item/id/${id}`)).json()).archived).toBe(false);
});
