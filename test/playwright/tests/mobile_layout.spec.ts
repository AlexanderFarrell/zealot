import type { APIRequestContext, Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { APP_SHELL, ITEM } from '../helpers/selectors';
import { uniqueTitle } from '../helpers/unique';

// Z195: at phone widths the right sidebar must not be drawn over the main
// content. Z165: it is its own pane, one tap from the top, instead of sitting
// at the bottom of a long scroll. Widths cover small Android through the
// iPhone Pro Max / Plus sizes.
const PHONE_WIDTHS = [360, 390, 430];
const PHONE_HEIGHT = 844;

const SCREENSHOT_DIR = process.env.Z195_SCREENSHOT_DIR;

/**
 * Asserts the pane switch sits in the screen's top button row, the side view
 * is hidden until its tab is tapped, and one tap shows it in the first
 * screenful with the content below the top rows out of the way.
 */
async function expectSidePaneOneTapAway(page: Page, label: RegExp, lowerContentSelector: string, screenshotName: string, iconButton = false): Promise<void> {
  const lowerContent = page.locator(lowerContentSelector).first();
  const sidebar = page.locator('right-sidebar');
  const strip = page.locator('mobile-pane-switch');
  const sideTab = iconButton ? strip.getByRole('button', { name: label }) : strip.getByRole('tab', { name: label });
  const stripEl = iconButton ? sideTab : strip;

  await expect(stripEl).toBeVisible();
  const stripBox = await stripEl.boundingBox();
  expect(stripBox!.y).toBeLessThan(200);
  expect(stripBox!.x + stripBox!.width).toBeLessThanOrEqual(page.viewportSize()!.width + 1);
  await expect(lowerContent).toBeVisible();
  await expect(sidebar).toBeHidden();

  await sideTab.click();
  await expect(sidebar.locator(':scope > *')).toHaveCount(1);
  await expect(sidebar).toBeVisible();
  await expect(lowerContent).toBeHidden();
  await expect(stripEl).toBeVisible();
  await expect(sideTab).toHaveAttribute(iconButton ? 'aria-pressed' : 'aria-selected', 'true');

  // No scrolling needed: the side view starts in the first screenful.
  expect(await page.evaluate(() => window.scrollY)).toBe(0);
  const sidebarBox = await sidebar.boundingBox();
  expect(sidebarBox!.y).toBeLessThan(PHONE_HEIGHT / 2);
  // No blank band left behind by the hidden content between the kept rows
  // (buttons, title, header, pill) and the side view.
  const keptBottom = await page.evaluate(() => {
    const kept = Array.from(document.querySelectorAll('center-content .mobile-pane-keep'));
    return Math.max(...kept.map((el) => el.getBoundingClientRect().bottom));
  });
  expect(sidebarBox!.y - keptBottom).toBeLessThanOrEqual(20);
  // Nothing wider than the phone, so no sideways scrolling.
  expect(sidebarBox!.x + sidebarBox!.width).toBeLessThanOrEqual(page.viewportSize()!.width + 1);

  // The document must be the only vertical scroller. A nested one-screen
  // scroller around the content traps touch scrolling on iOS Safari (Z195).
  const nestedScrollers = await page.evaluate(() => {
    const found: string[] = [];
    let el = document.querySelector('right-sidebar') as Element | null;
    for (; el && el !== document.body; el = el.parentElement) {
      const overflowY = getComputedStyle(el).overflowY;
      if ((overflowY === 'auto' || overflowY === 'scroll') && el.scrollHeight > el.clientHeight + 1) {
        found.push(`${el.tagName.toLowerCase()}${el.id ? '#' + el.id : ''}`);
      }
    }
    return found;
  });
  expect(nestedScrollers).toEqual([]);
  if (SCREENSHOT_DIR) {
    await page.screenshot({ path: `${SCREENSHOT_DIR}/${screenshotName}-side.png` });
  }
}

/** Creates an item with attributes (such as Parent) and returns its id. */
async function createItemWithAttributes(
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

    test('item screen puts Related one tap from the top', async ({ page, request }) => {
      // Enough children that the sidebar is taller than the phone screen,
      // the case that pushed it over the whole item in the Z195 report.
      const parentTitle = uniqueTitle('MobileParent');
      const parentId = await createItemWithAttributes(request, parentTitle, {});
      for (let i = 0; i < 15; i++) {
        await createItemWithAttributes(request, `${parentTitle}_child_${i}`, { Parent: [parentId] });
      }

      await page.goto(`/item/${encodeURIComponent(parentTitle)}`);
      await expect(page.locator(ITEM.title)).toContainText(parentTitle);
      await expect(page.locator('mobile-pane-switch').getByRole('tab', { name: 'Related (15)' })).toBeVisible();
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

      await expectSidePaneOneTapAway(page, /^Related \(15\)$/, 'item-screen .item-content', `item-${width}`);
    });

    test('daily planner puts Repeats one tap from the top', async ({ page, request }) => {
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
      await expect(page.locator('mobile-pane-switch').getByRole('button', { name: /^Repeats/ })).toBeVisible();
      await maybeScreenshot(page, `planner-${width}`);

      const header = await page.locator('daily-planner-screen .planner-header h1').boundingBox();
      const hit = await page.evaluate(({ x, y }) => {
        const el = document.elementFromPoint(x, y);
        return !!el?.closest('daily-planner-screen');
      }, { x: header!.x + 5, y: header!.y + header!.height / 2 });
      expect(hit).toBe(true);

      await expectSidePaneOneTapAway(page, /^Repeats/, 'daily-planner-screen .planner-section', `planner-${width}`, true);
    });

    test('planner header is two lines on every planner screen (Z239)', async ({ page }) => {
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
        expect(box!.height, path).toBeLessThan(90);
        const title = page.locator(`${screen} .planner-header h1`);
        expect((await title.boundingBox())!.height, path).toBeLessThan(40);
        // Every destination is still one tap away, scrolling row included.
        await expect(header.locator('button').first()).toBeAttached();
        const sizes = await header.locator('.planner-header-actions button').evaluateAll(
          els => els.map(el => { const r = el.getBoundingClientRect(); return `${Math.round(r.width)}x${Math.round(r.height)}`; }),
        );
        expect(new Set(sizes).size, `${path} button sizes ${sizes.join(',')}`).toBe(1);
        await maybeScreenshot(page, `planner-header-${screen}-${width}`);
      }
    });

    test('item attributes collapse and the toolbar waits for focus (Z240)', async ({ page, request }) => {
      const title = uniqueTitle('MobileClutter');
      await createItemWithAttributes(request, title, { Status: 'To Do', Priority: 3 },
        Array.from({ length: 60 }, (_, i) => `Line ${i}`).join('\n\n'));
      await page.goto(`/item/${encodeURIComponent(title)}`);
      await expect(page.locator(ITEM.title)).toContainText(title);

      const summary = page.locator('item-screen .attribute-summary');
      const table = page.locator('item-screen .attribute-editor');
      await expect(summary).toBeVisible();
      await expect(summary).toContainText('Status: To Do');
      await expect(table).toBeHidden();
      await expect(page.locator('item-screen .zealotscript-toolbar')).toBeHidden();

      const editor = page.locator('item-screen .ProseMirror');
      await expect(editor).toBeVisible();
      const box = await editor.boundingBox();
      expect(box!.y, 'content starts in the top half').toBeLessThan(PHONE_HEIGHT / 2);
      await maybeScreenshot(page, `item-collapsed-${width}`);

      await summary.click();
      await expect(table).toBeVisible();
      await expect(page.getByPlaceholder('New key')).toBeVisible();
      await summary.click();
      await expect(table).toBeHidden();

      await editor.click();
      const toolbar = page.locator('item-screen .zealotscript-toolbar');
      await expect(toolbar).toBeVisible();
      const toolbarBox = await toolbar.boundingBox();
      expect(toolbarBox!.x + toolbarBox!.width).toBeLessThanOrEqual(width + 1);
      // In flow above the text: it must never sit over the first lines.
      const textBox = await editor.boundingBox();
      expect(toolbarBox!.y + toolbarBox!.height).toBeLessThanOrEqual(textBox!.y + 1);
      await toolbar.getByRole('button').first().click();
      await expect(toolbar).toBeVisible();
    });

    test('attribute summary leads with a trimmed parent name (Z240)', async ({ page, request }) => {
      const parentTitle = `${uniqueTitle('MobileParentName')}_with_a_very_long_title_suffix`;
      const parentId = await createItemWithAttributes(request, parentTitle, {});
      const childTitle = uniqueTitle('MobileChild');
      await createItemWithAttributes(request, childTitle, { Parent: [parentId], Status: 'To Do' });
      await page.goto(`/item/${encodeURIComponent(childTitle)}`);

      const chips = page.locator('item-screen .attribute-summary-chip');
      await expect(chips.first()).toContainText('Parent: ');
      await expect(chips.first()).toContainText(parentTitle.slice(0, 20));
      await expect(chips.first()).toContainText('…');
      expect((await chips.first().innerText()).length).toBeLessThanOrEqual('Parent: '.length + 26);
    });

    test('type tables become cards with full-width titles (Z241)', async ({ page, request }) => {
      const title = `${uniqueTitle('MobileCard')} with a long wrapping title for a phone`;
      await createItemWithAttributes(request, title, { Schedule: '1111111', 'Time of Day': 'Morning' }, '', ['Repeat']);
      await page.goto('/types/Repeat');

      const row = page.locator('.item-table tbody tr:not(.item-table-create-row)').first();
      await expect(row).toBeVisible();
      expect(await row.evaluate((el) => getComputedStyle(el).display)).toBe('grid');
      const width = page.viewportSize()!.width;
      const titleBox = await row.locator('td[data-col-kind="title"]').boundingBox();
      expect(titleBox!.width).toBeGreaterThan(width * 0.6);
      expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(width + 1);
      // Attribute values keep their column name on the card.
      await expect(row.locator('td[data-col-kind="attribute"][data-label]').first()).toBeVisible();
    });

    test('repeats check off with one tap and fit one screen (Z238)', async ({ page, request }) => {
      const date = `2026-04-${String(width % 28 + 1).padStart(2, '0')}`;
      const prefix = uniqueTitle('MobileRepeat');
      for (let i = 0; i < 9; i++) {
        await createItemWithAttributes(
          request, `${prefix}_${i}`, { Schedule: '1111111', 'Time of Day': 'Morning' }, '', ['Repeat'],
        );
      }

      await page.goto(`/planner/daily/${date}`);
      const progress = page.locator('daily-planner-screen .planner-repeat-progress');
      await expect(progress).toBeVisible();
      await expect(progress).toHaveText(/^Repeats \d+\/\d+$/);
      const progressBox = await progress.boundingBox();
      expect(progressBox!.y).toBeLessThan(200);
      const before = Number((await progress.innerText()).match(/(\d+)\//)![1]);

      const paneButton = page.locator('mobile-pane-switch').getByRole('button', { name: /^Repeats/ });
      await paneButton.click();
      const rows = page.locator('right-sidebar .sidebar-repeat-row', { hasText: prefix });
      await expect(rows).toHaveCount(9);
      // Other widths create Repeats in parallel, so rows from several tests
      // interleave. Judge fit by row pitch: nine rows must use under 55% of the screen.
      const allRows = page.locator('right-sidebar .sidebar-repeat-row');
      const firstBox = await allRows.nth(0).boundingBox();
      const secondBox = await allRows.nth(1).boundingBox();
      const pitch = secondBox!.y - firstBox!.y;
      expect(pitch * 9).toBeLessThan(PHONE_HEIGHT * 0.55);
      const rowBox = await rows.first().boundingBox();
      expect(rowBox!.height).toBeLessThanOrEqual(44);
      expect(rowBox!.x + rowBox!.width).toBeLessThanOrEqual(width + 1);

      // One tap completes; a second clears it.
      const first = rows.first();
      const check = first.locator('.sidebar-repeat-check');
      await expect(check).toBeVisible();
      await check.click();
      await expect(first).toHaveClass(/is-complete/);
      await expect(check).toHaveAttribute('aria-pressed', 'true');

      await page.reload();
      // The chosen pane survives a reload, so only switch when Repeats is not showing.
      await expect(paneButton).toBeVisible();
      if (!(await page.locator('right-sidebar').isVisible())) await paneButton.click();
      await expect(page.locator('right-sidebar')).toBeVisible();
      await expect(rows.first()).toHaveClass(/is-complete/);
      await expect(page.locator('right-sidebar .sidebar-repeat-status').first()).toBeHidden();

      await rows.first().locator('.sidebar-repeat-check').click();
      await expect(rows.first()).not.toHaveClass(/is-complete/);

      // Skip is two taps: the more button, then Skip.
      await rows.first().locator('.sidebar-repeat-more').click();
      await rows.first().getByRole('button', { name: 'Skipped' }).click();
      await expect(rows.first()).toHaveAttribute('data-status', 'Skip');

      // A labelled button returns to the day's items.
      await expect(progress).toHaveText('← Items');
      await progress.click();
      await expect(progress).toBeVisible();
      await paneButton.click();
      await expect(page.locator('right-sidebar')).toBeVisible();

      // Progress strip tracks completions made in the side view.
      await rows.nth(1).locator('.sidebar-repeat-check').click();
      await expect(rows.nth(1)).toHaveClass(/is-complete/);
      await paneButton.click();
      await expect(progress).toHaveText(new RegExp(`^Repeats ${before + 1}/`));
    });

    test('Related stays selected while moving across items (Z165)', async ({ page, request }) => {
      const parentTitle = uniqueTitle('MobileSticky');
      const parentId = await createItemWithAttributes(request, parentTitle, {});
      const childTitle = `${parentTitle}_child`;
      const childId = await createItemWithAttributes(request, childTitle, { Parent: [parentId] });
      await createItemWithAttributes(request, `${childTitle}_grandchild`, { Parent: [childId] });

      await page.goto(`/item/${encodeURIComponent(parentTitle)}`);
      const strip = page.locator('mobile-pane-switch');
      await strip.getByRole('tab', { name: /^Related/ }).click();
      await page.locator('right-sidebar .item-card', { hasText: childTitle }).first().click();

      // The child opens on Related, with its own grandchild listed and no extra tap.
      await expect(page.locator('right-sidebar')).toContainText(`${childTitle}_grandchild`);
      await expect(strip.getByRole('tab', { name: /^Related/ })).toHaveAttribute('aria-selected', 'true');
      await expect(page.locator('item-screen .item-content')).toBeHidden();

      // Content switches back, and the next item then opens on Content.
      await strip.getByRole('tab', { name: 'Content' }).click();
      await expect(page.locator('item-screen .item-content')).toBeVisible();
      await page.goto(`/item/${encodeURIComponent(parentTitle)}`);
      await expect(strip.getByRole('tab', { name: 'Content' })).toHaveAttribute('aria-selected', 'true');
      await expect(page.locator('right-sidebar')).toBeHidden();
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
    await expect(page.locator('mobile-pane-switch')).toBeHidden();
    await maybeScreenshot(page, 'desktop-1440');
  });
});
