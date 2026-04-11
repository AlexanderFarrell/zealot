import { test, expect } from '../fixtures';
import { uniqueTitle } from '../helpers/unique';
import { SEARCH, APP_SHELL } from '../helpers/selectors';

// The app uses Ctrl+O (or Meta+O on macOS) to open the search modal
const CTRL_O = 'Control+o';

test.beforeEach(async ({ page }) => {
  await page.goto('/');
  await expect(page.locator(APP_SHELL.webClient)).toBeVisible({ timeout: 10_000 });
});

test('Ctrl+O opens the search modal', async ({ page }) => {
  await page.keyboard.press(CTRL_O);
  await expect(page.locator(SEARCH.modal)).toBeVisible();
  await expect(page.locator(SEARCH.input)).toBeFocused();
});

test('Escape closes the search modal', async ({ page }) => {
  await page.keyboard.press(CTRL_O);
  await expect(page.locator(SEARCH.modal)).toBeVisible();
  await expect(page.locator(SEARCH.input)).toBeFocused();

  await page.keyboard.press('Escape');
  await expect(page.locator(SEARCH.modal)).not.toBeVisible();
});

test('clicking the backdrop closes the search modal', async ({ page }) => {
  await page.keyboard.press(CTRL_O);
  await expect(page.locator(SEARCH.modal)).toBeVisible();

  // item-search-modal is the modal_background element itself.
  // Click in its top-left corner, outside the inner window.
  await page.locator(SEARCH.modal).click({ position: { x: 5, y: 5 } });
  await expect(page.locator(SEARCH.modal)).not.toBeVisible();
});

test('typing a search term shows matching results', async ({ page, createItem }) => {
  const title = uniqueTitle('SearchTarget');
  await createItem(title);

  await page.keyboard.press(CTRL_O);
  await expect(page.locator(SEARCH.input)).toBeVisible();

  // Type the distinctive prefix — the search debounce is 300ms
  await page.fill(SEARCH.input, 'SearchTarget');
  await expect(page.locator(SEARCH.resultRow).first()).toBeVisible({ timeout: 5_000 });
  await expect(page.locator(SEARCH.resultRow).first()).toContainText('SearchTarget');
});

test('clicking a search result navigates to the item and closes modal', async ({ page, createItem }) => {
  const title = uniqueTitle('ClickResult');
  await createItem(title);

  await page.keyboard.press(CTRL_O);
  await page.fill(SEARCH.input, 'ClickResult');
  await expect(page.locator(SEARCH.resultRow).first()).toBeVisible({ timeout: 5_000 });

  await page.locator(SEARCH.resultRow).first().click();

  // Modal should be gone and we should be on the item page
  await expect(page.locator(SEARCH.modal)).not.toBeVisible();
  // item-search-modal uses openItemById(), so URL uses /item_id/
  await expect(page).toHaveURL(/\/item_id\/\d+/);
});

test('keyboard navigation: ArrowDown + Enter navigates to item', async ({ page, createItem }) => {
  const title = uniqueTitle('KeyNavItem');
  await createItem(title);

  await page.keyboard.press(CTRL_O);
  await page.fill(SEARCH.input, 'KeyNavItem');
  await expect(page.locator(SEARCH.resultRow).first()).toBeVisible({ timeout: 5_000 });

  // Results auto-select index 0. ArrowDown moves to next (or stays at 0 if only one result).
  await page.keyboard.press('ArrowDown');
  await page.keyboard.press('Enter');

  await expect(page.locator(SEARCH.modal)).not.toBeVisible();
  await expect(page).toHaveURL(/\/item_id\/\d+/);
});

test('search with no results shows "No results." status', async ({ page }) => {
  await page.keyboard.press(CTRL_O);
  // Type a random string very unlikely to match any item
  await page.fill(SEARCH.input, 'ZZZZ_XYZ_IMPOSSIBLE_TITLE_9999');

  await expect(page.locator(SEARCH.status)).toBeVisible({ timeout: 5_000 });
  await expect(page.locator(SEARCH.status)).toContainText('No results.');
});
