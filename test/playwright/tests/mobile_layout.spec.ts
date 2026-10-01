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

  // The document must be the only vertical scroller. A nested one-screen
  // scroller around the content traps touch scrolling on iOS Safari, so the
  // stacked sidebar never comes into view there (Z195 follow-up).
  const nestedScrollers = await page.evaluate(() => {
    const found: string[] = [];
    let el = document.querySelector('right-sidebar')?.parentElement ?? null;
    const chain: Element[] = [];
    for (; el && el !== document.body; el = el.parentElement) chain.push(el);
    chain.push(document.querySelector('center-content')!);
    for (const node of chain) {
      const overflowY = getComputedStyle(node).overflowY;
      if ((overflowY === 'auto' || overflowY === 'scroll') && node.scrollHeight > node.clientHeight + 1) {
        found.push(`${node.tagName.toLowerCase()}${node.id ? '#' + node.id : ''}`);
      }
    }
    return found;
  });
  expect(nestedScrollers).toEqual([]);

  // Scroll the page itself (not an inner element) until the sidebar's top
  // reaches the middle of the screen, then check it is actually visible.
  await page.evaluate(() => {
    const top = document.querySelector('right-sidebar')!.getBoundingClientRect().top;
    window.scrollBy(0, top - window.innerHeight / 2);
  });
  expect(await page.evaluate(() => window.scrollY)).toBeGreaterThan(0);
  const scrolledBox = await sidebar.boundingBox();
  expect(scrolledBox!.y).toBeGreaterThanOrEqual(0);
  expect(scrolledBox!.y).toBeLessThan(PHONE_HEIGHT);
  await expect(sidebar.locator(':scope > *')).toBeInViewport();
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
    await page.evaluate(() => {
      window.scrollTo(0, 0);
      document.getElementById('main_web_ui')?.scrollTo(0, 0);
    });
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

      // Every toolbar action must be on screen; the row wraps instead of
      // being clipped by item-screen's overflow-x: hidden.
      const actions = page.locator('item-screen .button_row .item-action-btn');
      expect(await actions.count()).toBeGreaterThan(0);
      for (const box of await actions.evaluateAll(els => els.map(el => el.getBoundingClientRect().toJSON()))) {
        if (box.width === 0) continue;
        expect(box.x + box.width).toBeLessThanOrEqual(width + 1);
      }

      // The title must be the topmost thing the user can hit, not the sidebar.
      const title = await page.locator(ITEM.title).boundingBox();
      const hit = await page.evaluate(({ x, y }) => {
        const el = document.elementFromPoint(x, y);
        return !!el?.closest('item-screen');
      }, { x: title!.x + 5, y: title!.y + title!.height / 2 });
      expect(hit).toBe(true);

      await expectSidebarBelowContent(page, `item-${width}`);
    });

    test('daily planner keeps the Repeats sidebar below the planner', async ({ page, request }) => {
      // A full day makes the planner taller than the phone, so the sidebar can
      // only be reached by scrolling the page. Own date per width keeps other
      // planner specs' days untouched.
      const date = `2026-03-${String(width % 28 + 1).padStart(2, '0')}`;
      const prefix = uniqueTitle('MobileDay');
      for (let i = 0; i < 12; i++) {
        await createItemWithAttributes(request, `${prefix}_${i}`, { Date: date });
      }

      await page.goto(`/planner/daily/${date}`);
      await expect(page.locator('daily-planner-screen .planner-header h1')).toBeVisible();
      await expect(page.locator('daily-planner-screen .item-card').first()).toBeVisible();
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

    // Desktop keeps its panel scrollers; the mobile page-scroll rules must not leak.
    const scrollModel = await page.evaluate(() => ({
      main: getComputedStyle(document.getElementById('main_web_ui')!).overflowY,
      center: getComputedStyle(document.querySelector('center-content')!).overflowY,
      pageOverflows: document.scrollingElement!.scrollHeight > window.innerHeight,
    }));
    expect(scrollModel).toEqual({ main: 'auto', center: 'auto', pageOverflows: false });
    await maybeScreenshot(page, 'desktop-1440');
  });
});
