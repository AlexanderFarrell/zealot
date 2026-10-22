import { test, expect } from '../fixtures/own_account';
import { ITEM } from '../helpers/selectors';
import { uniqueTitle } from '../helpers/unique';

// Z269: Interface size and Start screen preferences (Settings → User).
test.use({ viewport: { width: 1440, height: 900 } });

async function choose(page: import('@playwright/test').Page, label: string, value: string): Promise<void> {
  await page.goto('/settings/user');
  await page.getByLabel(label).selectOption(value);
  // The settings store saves to the server after a short debounce.
  await page.waitForResponse((r) => r.url().endsWith('/api/account/settings') && r.request().method() === 'PATCH');
}

test('interface size zooms the whole app and survives a reload', async ({ page }) => {
  await choose(page, 'Interface size', 'larger');
  await expect.poll(() => page.evaluate(() => document.documentElement.style.zoom)).toBe('1.25');
  await page.reload();
  await expect.poll(() => page.evaluate(() => document.documentElement.style.zoom)).toBe('1.25');
  await choose(page, 'Interface size', 'default');
  await expect.poll(() => page.evaluate(() => document.documentElement.style.zoom)).toBe('');
});

test('the preference comes back from the server on a fresh browser profile', async ({ page }) => {
  await choose(page, 'Interface size', 'large');
  await page.evaluate(() => localStorage.clear());
  await page.reload();
  await expect.poll(() => page.evaluate(() => document.documentElement.style.zoom)).toBe('1.12');
});

test('start screen opens the daily planner at launch but never redirects a link', async ({ page, createOwnItem }) => {
  const title = uniqueTitle('StartDeepLink');
  await createOwnItem(title, 'content');
  await choose(page, 'Open on', 'daily');
  await page.goto('/');
  await expect(page).toHaveURL(/\/planner\/daily\/\d{4}-\d{2}-\d{2}$/);

  await page.goto(`/item/${encodeURIComponent(title)}`);
  await expect(page.locator(ITEM.title)).toHaveText(title);
  await expect(page).toHaveURL(new RegExp(`/item/${encodeURIComponent(title)}$`));
});

test('start screen can reopen the last item on this device', async ({ page, createOwnItem }) => {
  const title = uniqueTitle('StartLastItem');
  await createOwnItem(title, 'content');
  await page.goto(`/item/${encodeURIComponent(title)}`);
  await expect(page.locator(ITEM.title)).toHaveText(title);
  await choose(page, 'Open on', 'last_item');
  await page.goto(`/item/${encodeURIComponent(title)}`);
  await expect(page.locator(ITEM.title)).toHaveText(title);
  await page.goto('/');
  await expect(page).toHaveURL(/\/item_id\/\d+$/);
  await expect(page.locator(ITEM.title)).toHaveText(title);
});
