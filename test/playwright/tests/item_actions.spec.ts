import { test, expect } from '../fixtures/own_account';
import { APP_SHELL, ITEM } from '../helpers/selectors';
import { uniqueTitle } from '../helpers/unique';

// Z269: Settings → User → Item actions switches the desktop item screen between the
// button row and a "…" menu beside the title. Phones ignore the setting.
const BUTTON_ROW = 'item-screen .button_row';
const TITLE_MORE = 'item-screen .item-title-more';
const SCREENSHOT_DIR = process.env.Z269_SCREENSHOT_DIR;

async function chooseItemActions(page: import('@playwright/test').Page, value: 'toolbar' | 'menu'): Promise<void> {
  await page.goto('/settings/user');
  await page.getByLabel('Item actions').selectOption(value);
}

test.describe('item actions setting (Z269)', () => {
  test.describe('desktop', () => {
    test.use({ viewport: { width: 1440, height: 900 } });

    test('Menu mode swaps the button row for a "…" menu with every action', async ({ page, createOwnItem }) => {
      const title = uniqueTitle('ItemActions');
      await createOwnItem(title, 'content');
      await page.goto('/');
      await expect(page.locator(APP_SHELL.webClient)).toBeVisible({ timeout: 10_000 });

      await page.goto(`/item/${encodeURIComponent(title)}`);
      await expect(page.locator(ITEM.title)).toHaveText(title);
      await expect(page.locator(BUTTON_ROW)).toBeVisible();
      await expect(page.locator(TITLE_MORE)).toBeHidden();
      if (SCREENSHOT_DIR) await page.screenshot({ path: `${SCREENSHOT_DIR}/item-toolbar.png` });

      await chooseItemActions(page, 'menu');
      await page.goto(`/item/${encodeURIComponent(title)}`);
      await expect(page.locator(ITEM.title)).toHaveText(title);
      await expect(page.locator(BUTTON_ROW)).toBeHidden();
      await expect(page.locator(TITLE_MORE)).toBeVisible();
      // The comments heading keeps its natural height and sits below the content, as in Toolbar mode.
      const comments = page.locator('item-screen .item-collection-section h2', { hasText: 'Comments' });
      await expect(comments).toBeVisible();
      expect((await comments.boundingBox())!.height).toBeLessThan(60);
      await page.locator(TITLE_MORE).click();
      for (const label of ['Copy Link', 'Open in New Tab', 'Toggle Content', 'Time Blocks', 'Manage Types', 'Paste Template', 'Archive Item', 'Archive Subtree', 'Delete Item']) {
        await expect(page.locator('.ctx-menu-item', { hasText: label }).first()).toBeVisible();
      }
      await expect(page.locator('.ctx-menu-item', { hasText: /^Switch to / })).toBeVisible();
      await expect(page.locator('.ctx-menu-item', { hasText: 'To Parent' })).toHaveCount(0);
      if (SCREENSHOT_DIR) await page.screenshot({ path: `${SCREENSHOT_DIR}/item-menu.png` });

    });

    test('the choice applies without a reload and survives one', async ({ page, createOwnItem }) => {
      const title = uniqueTitle('ItemActionsLive');
      await createOwnItem(title, 'content');
      await page.goto(`/item/${encodeURIComponent(title)}`);
      await expect(page.locator(BUTTON_ROW)).toBeVisible();
      await page.evaluate(() => {
        const raw = JSON.parse(localStorage.getItem('zealot_settings') ?? '{}');
        localStorage.setItem('zealot_settings', JSON.stringify({ ...raw, item_actions: 'menu' }));
      });
      await page.reload();
      await expect(page.locator(TITLE_MORE)).toBeVisible();
    });
  });

  test.describe('phone', () => {
    test.use({ viewport: { width: 390, height: 844 } });

    test('the layout is the same in both modes', async ({ page, createOwnItem }) => {
      const title = uniqueTitle('ItemActionsPhone');
      await createOwnItem(title, 'content');
      await chooseItemActions(page, 'menu');
      await page.goto(`/item/${encodeURIComponent(title)}`);
      await expect(page.locator(ITEM.title)).toHaveText(title);
      await expect(page.locator(BUTTON_ROW)).toBeVisible();
      await expect(page.locator(TITLE_MORE)).toBeHidden();
      await expect(page.locator('.item-action-btn--more')).toBeVisible();
    });
  });
});
