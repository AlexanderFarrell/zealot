import type { APIRequestContext, Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { APP_SHELL, ITEM } from '../helpers/selectors';
import { uniqueTitle } from '../helpers/unique';

// Z253: the compact Repeats, planner header and item attribute designs from
// Z238-Z240 on a 1440x900 desktop window.
const WIDTH = 1440;
const HEIGHT = 900;
const SCREENSHOT_DIR = process.env.Z253_SCREENSHOT_DIR;

async function createItem(
  request: APIRequestContext,
  title: string,
  attributes: Record<string, unknown>,
  content = '',
  types: string[] = [],
): Promise<number> {
  const csrf = (await request.storageState()).cookies.find(cookie => cookie.name === 'csfr_')?.value;
  const res = await request.post('/api/item', {
    data: { title, content, attributes, ...(types.length > 0 ? { types } : {}) },
    headers: csrf ? { 'X-Csrf-Token': csrf } : undefined,
  });
  expect(res.ok(), await res.text()).toBe(true);
  return (await res.json() as { item_id: number }).item_id;
}

async function maybeScreenshot(page: Page, name: string): Promise<void> {
  if (SCREENSHOT_DIR) await page.screenshot({ path: `${SCREENSHOT_DIR}/${name}.png` });
}

test.describe(`desktop viewport ${WIDTH}x${HEIGHT} (Z253)`, () => {
  test.use({ viewport: { width: WIDTH, height: HEIGHT } });

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator(APP_SHELL.webClient)).toBeVisible({ timeout: 10_000 });
  });

  test('item content starts in the top third and attributes stay editable', async ({ page, request }) => {
    const parentTitle = uniqueTitle('DesktopParent');
    const parentId = await createItem(request, parentTitle, {});
    const secondTitle = uniqueTitle('DesktopSecondParent');
    const secondId = await createItem(request, secondTitle, {});
    const title = uniqueTitle('DesktopClutter');
    await createItem(request, title, { Status: 'To Do', Priority: 3, Parent: [parentId, secondId] },
      Array.from({ length: 60 }, (_, i) => `Line ${i}`).join('\n\n'));
    await page.goto(`/item/${encodeURIComponent(title)}`);
    await expect(page.locator(ITEM.title)).toContainText(title);

    const summary = page.locator('item-screen .attribute-summary');
    const table = page.locator('item-screen .attribute-editor');
    await expect(summary).toBeVisible();
    await expect(summary).toContainText('Status: To Do');
    await expect(table).toBeHidden();
    await expect(page.getByPlaceholder('New key')).toBeHidden();

    const toolbar = page.locator('item-screen .zealotscript-toolbar');
    await expect(toolbar).toBeVisible();
    const toolbarBox = await toolbar.boundingBox();
    expect(toolbarBox!.height, 'toolbar is one line').toBeLessThan(44);

    const editor = page.locator('item-screen .ProseMirror');
    const editorBox = await editor.boundingBox();
    expect(editorBox!.y, 'content starts in the top third').toBeLessThan(HEIGHT / 3);
    await maybeScreenshot(page, 'item-collapsed');

    // Expanding is one click, remembered across items, and keeps New key available.
    // The caret toggles; the Parent chips in the middle navigate instead.
    await summary.locator('.attribute-summary-caret').click();
    await expect(table).toBeVisible();
    await expect(page.getByPlaceholder('New key')).toBeVisible();
    await maybeScreenshot(page, 'item-expanded');

    const other = uniqueTitle('DesktopOther');
    await createItem(request, other, { Status: 'To Do' });
    await page.goto(`/item/${encodeURIComponent(other)}`);
    await expect(page.locator(ITEM.title)).toContainText(other);
    await expect(page.locator('item-screen .attribute-editor')).toBeVisible();

    await page.locator('item-screen .attribute-summary').click();
    await expect(page.locator('item-screen .attribute-editor')).toBeHidden();
    await page.goto(`/item/${encodeURIComponent(title)}`);
    await expect(page.locator(ITEM.title)).toContainText(title);
    await expect(page.locator('item-screen .attribute-editor')).toBeHidden();

    // Every parent has its own chip, and each navigates without expanding.
    const chips = page.locator('item-screen .attribute-summary-chip--link');
    await expect(chips).toHaveCount(2);
    await expect(chips.nth(1)).toContainText(secondTitle.slice(0, 20));
    await chips.nth(1).click();
    await expect(page.locator(ITEM.title)).toContainText(secondTitle);
    await page.goBack();
    await expect(page.locator(ITEM.title)).toContainText(title);
    await page.locator('item-screen .attribute-summary-chip--link', { hasText: 'Parent:' }).click();
    await expect(page.locator(ITEM.title)).toContainText(parentTitle);
  });

  test('planner header is one row on every planner screen', async ({ page }) => {
    for (const [path, screen] of [
      ['/planner/daily/2026-03-05', 'daily-planner-screen'],
      ['/planner/weekly/2026-W10', 'weekly-planner-screen'],
      ['/planner/monthly/2026-03', 'monthly-planner-screen'],
      ['/planner/annual/2026', 'annual-planner-screen'],
    ] as const) {
      await page.goto(path);
      const header = page.locator(`${screen} .planner-header`);
      await expect(header).toBeVisible();
      const box = await header.boundingBox();
      expect(box!.height, `${path} header is one row`).toBeLessThan(44);
      const buttons = header.locator('.planner-header-actions button');
      for (const label of await buttons.evaluateAll(els => els.map(el => el.getAttribute('title')))) {
        expect(label, `${path} chips carry a title`).toBeTruthy();
      }
      await maybeScreenshot(page, `planner-header-${screen}`);
    }
  });

  test('planner chips drop their text when the column is narrow', async ({ page }) => {
    await page.goto('/planner/daily/2026-03-05');
    const chip = page.locator('daily-planner-screen .planner-header-link').first();
    await expect(chip).toBeVisible();
    await expect(chip.locator('span')).toBeHidden();
    await expect(chip).toHaveAttribute('title', 'This Week');
  });

  test('repeats check off with one click and the rest sit behind the menu', async ({ page, request }) => {
    const prefix = uniqueTitle('DesktopRepeat');
    for (let i = 0; i < 3; i++) {
      await createItem(request, `${prefix}_${i}`, { Schedule: '1111111', 'Time of Day': 'Morning' }, '', ['Repeat']);
    }
    await page.goto('/planner/daily/2026-04-09');
    const rows = page.locator('right-sidebar .sidebar-repeat-row', { hasText: prefix });
    await expect(rows).toHaveCount(3);
    await expect(page.locator('right-sidebar .sidebar-repeat-status').first()).toBeHidden();
    await maybeScreenshot(page, 'repeats');

    const first = rows.first();
    const check = first.locator('.sidebar-repeat-check');
    await check.click();
    await expect(first).toHaveClass(/is-complete/);
    await expect(check).toHaveAttribute('aria-pressed', 'true');
    await check.click();
    await expect(first).not.toHaveClass(/is-complete/);

    await first.hover();
    await first.locator('.sidebar-repeat-more').click();
    await first.getByRole('button', { name: 'Skipped' }).click();
    await expect(first).toHaveAttribute('data-status', 'Skip');
  });
});
