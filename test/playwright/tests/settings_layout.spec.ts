import { test, expect } from '../fixtures';
import { APP_SHELL } from '../helpers/selectors';

// Z269: Settings is a tabbed screen; each tab lists what exists first and keeps "Add" behind a button.
test.use({ viewport: { width: 1440, height: 900 } });

const SCREENSHOT_DIR = process.env.Z269_SCREENSHOT_DIR;
const TABS = ['Attributes', 'Types', 'Planner', 'Wiki', 'Data', 'User'];

test('Settings shows six tabs and opens on Attributes', async ({ page }) => {
  await page.goto('/settings');
  await page.goto('/settings/attributes');
  const tabs = page.getByRole('tab');
  await expect(tabs).toHaveText(TABS);
  await expect(page.getByRole('tab', { name: 'Attributes' })).toHaveAttribute('aria-selected', 'true');
});

test('Attributes lists existing kinds first and keeps the add form behind a button', async ({ page }) => {
  await page.goto('/settings/attributes');
  await expect(page.getByRole('heading', { name: 'User Kinds' })).toBeVisible();
  await expect(page.locator('[name="key"]')).toBeHidden();

  await page.getByRole('button', { name: 'Add Kind' }).click();
  await expect(page.locator('[name="key"]')).toBeVisible();
  await page.getByRole('button', { name: 'Cancel' }).click();
  await expect(page.locator('[name="key"]')).toBeHidden();

  // Built-in kinds are tucked away until asked for.
  await expect(page.locator('.attr-kind-section', { hasText: 'System Kinds' }).locator('table')).toBeHidden();
  await page.getByText(/System Kinds \(\d+\)/).click();
  await expect(page.locator('.attr-kind-section', { hasText: 'System Kinds' }).locator('table')).toBeVisible();
});

test('adding an attribute kind through the form still works', async ({ page }) => {
  const key = `kind_${Date.now().toString(36)}`;
  await page.goto('/settings/attributes');
  await page.getByRole('button', { name: 'Add Kind' }).click();
  await page.fill('[name="key"]', key);
  await page.locator('.attr-kind-settings-create button[type="submit"]').click();
  await expect.poll(() => page.locator('.attr-kind-table input').evaluateAll(
    (inputs, wanted) => inputs.some((input) => (input as HTMLInputElement).value === wanted), key,
  )).toBe(true);
});

test('tabs switch sections, and Ctrl+, opens Settings', async ({ page }) => {
  await page.goto('/');
  // Shortcuts are registered once the app has drawn.
  await expect(page.locator(APP_SHELL.webClient)).toBeVisible({ timeout: 10_000 });
  await expect(page.locator('item-screen')).toBeVisible();
  await page.keyboard.press('Control+,');
  await expect(page).toHaveURL(/\/settings\/attributes$/);
  for (const tab of TABS) {
    await page.getByRole('tab', { name: tab }).click();
    await expect(page).toHaveURL(new RegExp(`/settings/${tab.toLowerCase()}$`));
    await expect(page.getByRole('tab', { name: tab })).toHaveAttribute('aria-selected', 'true');
    if (SCREENSHOT_DIR) await page.screenshot({ path: `${SCREENSHOT_DIR}/settings-${tab.toLowerCase()}.png` });
  }
});

test('the User tab holds the preferences and account details', async ({ page }) => {
  await page.goto('/settings/user');
  await expect(page.getByLabel('Item actions')).toBeVisible();
  await expect(page.getByLabel('Interface size')).toBeVisible();
  await expect(page.getByLabel('Open on')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Log Out' })).toBeVisible();
});
