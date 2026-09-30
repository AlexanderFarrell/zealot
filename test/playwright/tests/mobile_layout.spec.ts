import type { APIRequestContext, Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { APP_SHELL, ITEM } from '../helpers/selectors';
import { uniqueTitle } from '../helpers/unique';

// Z195: at phone widths the right sidebar must stack below the main content
// instead of being drawn over it. Widths cover small Android through the
// iPhone Pro Max / Plus sizes.
const PHONE_WIDTHS = [360, 390, 430];
const PHONE_HEIGHT = 844;

const SCREENSHOT_DIR = process.env.Z195_SCREENSHOT_DIR;

/**
 * Asserts the right sidebar has content, sits entirely below the center
 * content, and can be scrolled into view.
 */
async function expectSidebarBelowContent(page: Page, screenshotName: string): Promise<void> {
  const center = page.locator('center-content');
  const sidebar = page.locator('right-sidebar');
  await expect(sidebar.locator(':scope > *')).toHaveCount(1);

  const centerBox = await center.boundingBox();
  const sidebarBox = await sidebar.boundingBox();
  expect(centerBox).not.toBeNull();
  expect(sidebarBox).not.toBeNull();
  expect(sidebarBox!.y).toBeGreaterThanOrEqual(centerBox!.y + centerBox!.height - 1);

  // Nothing wider than the phone, so no sideways scrolling to reach content.
  expect(sidebarBox!.x + sidebarBox!.width).toBeLessThanOrEqual(page.viewportSize()!.width + 1);

  await sidebar.scrollIntoViewIfNeeded();
  await expect(sidebar).toBeInViewport();
  if (SCREENSHOT_DIR) {
    await page.screenshot({ path: `${SCREENSHOT_DIR}/${screenshotName}-sidebar.png` });
  }
}

/** Creates an item with attributes (such as Parent) and returns its id. */
async function createItemWithAttributes(
  request: APIRequestContext,
  title: string,
  attributes: Record<string, unknown>,
): Promise<number> {
  const csrf = (await request.storageState()).cookies.find(cookie => cookie.name === 'csfr_')?.value;
  const res = await request.post('/api/item', {
    data: { title, content: '', attributes },
    headers: csrf ? { 'X-Csrf-Token': csrf } : undefined,
  });
  expect(res.ok(), await res.text()).toBe(true);
  return (await res.json() as { item_id: number }).item_id;
}

/** Saves a screenshot from the top of the page when Z195_SCREENSHOT_DIR is set. */
async function maybeScreenshot(page: Page, name: string): Promise<void> {
  if (SCREENSHOT_DIR) {
    await page.evaluate(() => document.getElementById('main_web_ui')?.scrollTo(0, 0));
    await page.screenshot({ path: `${SCREENSHOT_DIR}/${name}.png` });
  }
}

for (const width of PHONE_WIDTHS) {
  test.describe(`phone viewport ${width}px`, () => {
    test.use({ viewport: { width, height: PHONE_HEIGHT } });

    test.beforeEach(async ({ page }) => {
      await page.goto('/');
      await expect(page.locator(APP_SHELL.webClient)).toBeVisible({ timeout: 10_000 });
    });

    test('item screen keeps the Children sidebar below the item', async ({ page, request }) => {
      // Enough children that the sidebar is taller than the phone screen,
      // the case that pushed it over the whole item in the Z195 report.
      const parentTitle = uniqueTitle('MobileParent');
      const parentId = await createItemWithAttributes(request, parentTitle, {});
      for (let i = 0; i < 15; i++) {
        await createItemWithAttributes(request, `${parentTitle}_child_${i}`, { Parent: [parentId] });
      }

      await page.goto(`/item/${encodeURIComponent(parentTitle)}`);
      await expect(page.locator(ITEM.title)).toContainText(parentTitle);
      await expect(page.locator('right-sidebar .item-card').first()).toBeVisible();
      await expect(page.locator('right-sidebar .item-collections')).toBeVisible();
      await maybeScreenshot(page, `item-${width}`);

      // The title must be the topmost thing the user can hit, not the sidebar.
      const title = await page.locator(ITEM.title).boundingBox();
      const hit = await page.evaluate(({ x, y }) => {
        const el = document.elementFromPoint(x, y);
        return !!el?.closest('item-screen');
      }, { x: title!.x + 5, y: title!.y + title!.height / 2 });
      expect(hit).toBe(true);

      await expectSidebarBelowContent(page, `item-${width}`);
    });

    test('daily planner keeps the Repeats sidebar below the planner', async ({ page }) => {
      await page.goto('/planner/daily/2026-01-15');
      await expect(page.locator('daily-planner-screen .planner-header h1')).toBeVisible();
      await expect(page.locator('right-sidebar > *')).toHaveCount(1);
      await maybeScreenshot(page, `planner-${width}`);

      const header = await page.locator('daily-planner-screen .planner-header h1').boundingBox();
      const hit = await page.evaluate(({ x, y }) => {
        const el = document.elementFromPoint(x, y);
        return !!el?.closest('daily-planner-screen');
      }, { x: header!.x + 5, y: header!.y + header!.height / 2 });
      expect(hit).toBe(true);

      await expectSidebarBelowContent(page, `planner-${width}`);
    });
  });
}

test.describe('desktop viewport', () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test('right sidebar stays a column beside the content', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator(ITEM.title)).toBeVisible();
    const center = await page.locator('center-content').boundingBox();
    const sidebar = await page.locator('right-sidebar').boundingBox();
    expect(sidebar!.x).toBeGreaterThanOrEqual(center!.x + center!.width);
    expect(Math.abs(sidebar!.y - center!.y)).toBeLessThanOrEqual(1);
    await maybeScreenshot(page, 'desktop-1440');
  });
});
