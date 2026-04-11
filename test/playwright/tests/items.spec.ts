import { test, expect } from '../fixtures';
import { uniqueTitle } from '../helpers/unique';
import { ADD_ITEM, ITEM, CONFIRM, APP_SHELL } from '../helpers/selectors';

// On macOS Playwright sends Control+key even if the app uses Cmd on real hardware.
// The app's hotkey registration uses keyboard shortcuts that Playwright handles uniformly.
const CTRL_N = 'Control+n';
const CTRL_O = 'Control+o';

test.beforeEach(async ({ page }) => {
  await page.goto('/');
  await expect(page.locator(APP_SHELL.webClient)).toBeVisible({ timeout: 10_000 });
});

test('home page loads the Home item', async ({ page }) => {
  await expect(page.locator(ITEM.screen)).toBeVisible();
  await expect(page.locator(ITEM.title)).toContainText('Home');
});

test('Ctrl+N opens the add-item modal', async ({ page }) => {
  await page.keyboard.press(CTRL_N);
  await expect(page.locator(ADD_ITEM.modal)).toBeVisible();
  await expect(page.locator(ADD_ITEM.titleInput)).toBeFocused();
});

test('Escape closes the add-item modal', async ({ page }) => {
  await page.keyboard.press(CTRL_N);
  await expect(page.locator(ADD_ITEM.modal)).toBeVisible();
  await expect(page.locator(ADD_ITEM.titleInput)).toBeFocused();

  await page.keyboard.press('Escape');
  await expect(page.locator(ADD_ITEM.modal)).not.toBeVisible();
});

test('Cancel button closes the add-item modal', async ({ page }) => {
  await page.keyboard.press(CTRL_N);
  await expect(page.locator(ADD_ITEM.modal)).toBeVisible();

  await page.click(ADD_ITEM.cancelButton);
  await expect(page.locator(ADD_ITEM.modal)).not.toBeVisible();
});

test('create item via Ctrl+N modal navigates to the new item', async ({ page }) => {
  const title = uniqueTitle('NewItem');

  await page.keyboard.press(CTRL_N);
  await expect(page.locator(ADD_ITEM.modal)).toBeVisible();

  await page.fill(ADD_ITEM.titleInput, title);
  await page.click(ADD_ITEM.submitButton);

  await expect(page.locator(ADD_ITEM.modal)).not.toBeVisible();
  await expect(page.locator(ITEM.screen)).toBeVisible();
  await expect(page.locator(ITEM.title)).toContainText(title);
  await expect(page).toHaveURL(/\/item\//);
});

test('create item with blank title shows error', async ({ page }) => {
  await page.keyboard.press(CTRL_N);
  await expect(page.locator(ADD_ITEM.modal)).toBeVisible();

  await page.click(ADD_ITEM.submitButton);

  await expect(page.locator(ADD_ITEM.errorEl)).toBeVisible();
  await expect(page.locator(ADD_ITEM.errorEl)).toContainText('Title is required.');
  await expect(page.locator(ADD_ITEM.modal)).toBeVisible();
});

test('view item by navigating to /item/{title}', async ({ page, createItem }) => {
  const title = uniqueTitle('ViewMe');
  await createItem(title);

  await page.goto(`/item/${encodeURIComponent(title)}`);
  await expect(page.locator(ITEM.screen)).toBeVisible();
  await expect(page.locator(ITEM.title)).toHaveText(title);
});

test('edit item title persists after reload', async ({ page, createItem }) => {
  const title = uniqueTitle('EditTitle');
  const newTitle = uniqueTitle('Renamed');
  await createItem(title);

  await page.goto(`/item/${encodeURIComponent(title)}`);
  await expect(page.locator(ITEM.title)).toBeVisible();

  // Triple-click selects all text in a contenteditable more reliably than Ctrl+A
  await page.locator(ITEM.title).click({ clickCount: 3 });
  await page.keyboard.type(newTitle);
  // Blur triggers the PATCH
  await page.locator(ITEM.title).blur();
  await page.waitForTimeout(500);

  await page.reload();
  await expect(page.locator(ITEM.title)).toHaveText(newTitle, { timeout: 10_000 });
});

test('edit item content persists after reload', async ({ page, createItem }) => {
  const title = uniqueTitle('EditContent');
  await createItem(title);
  const content = `Some notes about ${title}`;

  await page.goto(`/item/${encodeURIComponent(title)}`);
  await expect(page.locator(ITEM.screen)).toBeVisible();

  const textarea = page.locator('item-screen .item-content textarea');
  await textarea.fill(content);
  // Content save is debounced 1000ms — wait for it to fire
  await page.waitForTimeout(1500);

  await page.reload();
  await expect(page.locator('item-screen .item-content textarea')).toHaveValue(content, { timeout: 10_000 });
});

test('item not found shows descriptive message', async ({ page }) => {
  const ghost = `ZZZ_ghost_${Date.now()}`;
  await page.goto(`/item/${ghost}`);

  await expect(page.locator(ITEM.notFoundMsg)).toBeVisible();
});

test('"Create it?" button appears when item not found by title', async ({ page }) => {
  const ghost = `ZZZ_ghost_${Date.now()}`;
  await page.goto(`/item/${ghost}`);

  await expect(page.locator(ITEM.createItBtn)).toBeVisible();
});

test('delete item navigates away', async ({ page, createItem }) => {
  const title = uniqueTitle('DeleteMe');
  await createItem(title);

  await page.goto(`/item/${encodeURIComponent(title)}`);
  await expect(page.locator(ITEM.screen)).toBeVisible();

  // Click the Delete Item button (icon button with title="Delete Item")
  await page.click('button[title="Delete Item"]');

  // ConfirmDialog uses Shadow DOM — Playwright pierces open shadow roots
  const confirmBtn = page.locator(CONFIRM.confirmButton);
  await expect(confirmBtn).toBeVisible();
  await confirmBtn.click();

  // After deletion the navigator goes to parent or home
  await expect(page.locator(ITEM.screen)).toBeVisible({ timeout: 10_000 });
  // Should no longer be on the deleted item's page
  await expect(page.locator(ITEM.title)).not.toHaveText(title, { timeout: 5_000 });
});
