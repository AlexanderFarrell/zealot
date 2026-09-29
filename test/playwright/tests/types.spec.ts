import { test, expect } from '../fixtures';
import { uniqueTitle, generateSuffix } from '../helpers/unique';
import { APP_SHELL, CONFIRM } from '../helpers/selectors';

test.beforeEach(async ({ page }) => {
  await page.goto('/');
  await expect(page.locator(APP_SHELL.webClient)).toBeVisible({ timeout: 10_000 });
});

test('/types renders the types screen with heading', async ({ page }) => {
  await page.goto('/types');
  await expect(page.locator('types-screen')).toBeVisible();
  await expect(page.locator('types-screen h1')).toHaveText('Types');
});

test('"Create Type" button on types screen navigates to /settings/types', async ({ page }) => {
  await page.goto('/types');
  await expect(page.locator('types-screen')).toBeVisible();

  await page.click('types-screen button:text("Create Type")');
  await expect(page).toHaveURL('/settings/types');
  await expect(page.locator('settings-screen')).toBeVisible();
});

test('/settings/types shows the type creation form', async ({ page }) => {
  await page.goto('/settings/types');
  await expect(page.locator('settings-screen')).toBeVisible();
  await expect(page.locator('[name="type_name"]')).toBeVisible();
  await expect(page.locator('.type-settings-create button[type="submit"]')).toBeVisible();
});

test('create a new type via settings and navigate to its type screen', async ({ page }) => {
  const typeName = `Type_${generateSuffix()}`;

  await page.goto('/settings/types');
  await expect(page.locator('[name="type_name"]')).toBeVisible();

  await page.fill('[name="type_name"]', typeName);
  await page.click('.type-settings-create button[type="submit"]');

  // After creation, openType() navigates to /types/{name}
  await expect(page).toHaveURL(new RegExp(`/types/${encodeURIComponent(typeName)}`), { timeout: 10_000 });
  await expect(page.locator('type-screen')).toBeVisible();
});

test('create type with blank name shows error', async ({ page }) => {
  await page.goto('/settings/types');
  await expect(page.locator('[name="type_name"]')).toBeVisible();

  // Submit without filling the name
  await page.click('.type-settings-create button[type="submit"]');

  // type_settings_screen.ts sets createError = 'Type name is required.'
  await expect(page.locator('.tool-error')).toContainText('Type name is required.');
});

test('types screen table row click navigates to type detail', async ({ page, request }) => {
  // Create a type via API first so we know there's at least one
  const typeName = `Type_${generateSuffix()}`;
  await request.post('/api/item_type', {
    data: { name: typeName, description: '', required_attributes: [] },
  });

  await page.goto('/types');
  await expect(page.locator('types-screen')).toBeVisible();

  // Wait for the table to load and find our new type row
  const typeButton = page.locator('.type-row-link', { hasText: typeName });
  await expect(typeButton).toBeVisible({ timeout: 10_000 });
  await typeButton.click();

  await expect(page).toHaveURL(new RegExp(`/types/${encodeURIComponent(typeName)}`));
  await expect(page.locator('type-screen')).toBeVisible();
});

test('delete a type from settings', async ({ page, request }) => {
  const typeName = `DelType_${generateSuffix()}`;
  await request.post('/api/item_type', {
    data: { name: typeName, description: '', required_attributes: [] },
  });

  await page.goto('/settings/types');
  await expect(page.locator('settings-screen')).toBeVisible();

  // Wait for the type to appear in the list
  const row = page.locator('.types-summary-table tbody tr', { hasText: typeName });
  await expect(row).toBeVisible({ timeout: 10_000 });

  await row.locator('button:text("Delete")').click();

  // ConfirmDialog uses shadow DOM — Playwright pierces open shadow roots
  await expect(page.locator(CONFIRM.dialog)).toBeVisible();
  await page.locator(CONFIRM.confirmButton).click();

  // Row should be gone after deletion
  await expect(row).not.toBeVisible({ timeout: 10_000 });
});
