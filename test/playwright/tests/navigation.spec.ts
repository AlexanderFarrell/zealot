import { test, expect } from '../fixtures';
import { SIDEBAR, ITEM, APP_SHELL } from '../helpers/selectors';

test.beforeEach(async ({ page }) => {
  await page.goto('/');
  await expect(page.locator(APP_SHELL.webClient)).toBeVisible({ timeout: 10_000 });
});

test('/types renders types screen', async ({ page }) => {
  await page.goto('/types');
  await expect(page.locator('types-screen')).toBeVisible();
});

test('/settings/attributes renders settings screen', async ({ page }) => {
  await page.goto('/settings/attributes');
  await expect(page.locator('settings-screen')).toBeVisible();
});

test('/planner/daily/2026-01-15 renders daily planner screen', async ({ page }) => {
  await page.goto('/planner/daily/2026-01-15');
  await expect(page.locator('daily-planner-screen')).toBeVisible();
});

test('/planner/daily (no date) redirects to today', async ({ page }) => {
  await page.goto('/planner/daily');
  await expect(page).toHaveURL(/\/planner\/daily\/\d{4}-\d{2}-\d{2}/, { timeout: 5_000 });
  await expect(page.locator('daily-planner-screen')).toBeVisible();
});

test('/planner/weekly (no date) redirects to this week', async ({ page }) => {
  await page.goto('/planner/weekly');
  await expect(page).toHaveURL(/\/planner\/weekly\/\d{4}-W\d{2}/, { timeout: 5_000 });
  await expect(page.locator('weekly-planner-screen')).toBeVisible();
});

test('/planner/monthly (no date) redirects to this month', async ({ page }) => {
  await page.goto('/planner/monthly');
  await expect(page).toHaveURL(/\/planner\/monthly\/\d{4}-\d{2}/, { timeout: 5_000 });
  await expect(page.locator('monthly-planner-screen')).toBeVisible();
});

test('unknown path shows 404 message', async ({ page }) => {
  await page.goto('/this/path/does/not/exist');
  // web_navigator.ts returns a div with text "404 Not Found: {path}"
  await expect(page.locator('text=404 Not Found:')).toBeVisible();
});

test('sidebar "Home Page" button navigates to /', async ({ page }) => {
  await page.goto('/types');
  await expect(page.locator('types-screen')).toBeVisible();

  await page.click(SIDEBAR.home);
  await expect(page).toHaveURL('/');
  await expect(page.locator(ITEM.screen)).toBeVisible();
});

test('sidebar "Daily Planner" button navigates to daily planner', async ({ page }) => {
  await page.click(SIDEBAR.dailyPlanner);
  await expect(page).toHaveURL(/\/planner\/daily\//);
  await expect(page.locator('daily-planner-screen')).toBeVisible();
});

test('sidebar "Weekly Planner" button navigates to weekly planner', async ({ page }) => {
  await page.click(SIDEBAR.weeklyPlanner);
  await expect(page).toHaveURL(/\/planner\/weekly\//);
  await expect(page.locator('weekly-planner-screen')).toBeVisible();
});

test('browser back button works after navigation', async ({ page }) => {
  await page.goto('/');
  await page.goto('/types');
  await expect(page.locator('types-screen')).toBeVisible();

  await page.goBack();
  await expect(page).toHaveURL('/');
  await expect(page.locator(ITEM.screen)).toBeVisible();
});

test('/item/{title} URL loads the item screen', async ({ page, createItem }) => {
  const { uniqueTitle } = await import('../helpers/unique');
  const title = uniqueTitle('NavItem');
  await createItem(title);

  await page.goto(`/item/${encodeURIComponent(title)}`);
  await expect(page.locator(ITEM.screen)).toBeVisible();
  await expect(page.locator(ITEM.title)).toHaveText(title);
});
