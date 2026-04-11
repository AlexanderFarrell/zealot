import { test, expect } from '../fixtures';
import { COMMENTS, SIDEBAR, APP_SHELL } from '../helpers/selectors';

test.beforeEach(async ({ page }) => {
  await page.goto('/');
  await expect(page.locator(APP_SHELL.webClient)).toBeVisible({ timeout: 10_000 });
});

test('daily planner renders sections: Items, Repeats, Comments', async ({ page }) => {
  await page.goto('/planner/daily/2026-01-15');
  await expect(page.locator('daily-planner-screen')).toBeVisible();

  // createPlannerSection creates <section class="planner-section"><h2>Title</h2>
  await expect(page.locator('.planner-section h2:text("Items")')).toBeVisible();
  await expect(page.locator('.planner-section h2:text("Repeats")')).toBeVisible();
  await expect(page.locator('.planner-section h2:text("Comments")')).toBeVisible();
});

test('daily planner shows formatted date in header', async ({ page }) => {
  // 2026-01-15 is a Thursday
  await page.goto('/planner/daily/2026-01-15');
  await expect(page.locator('daily-planner-screen')).toBeVisible();

  // formatDayTitle returns "Thursday, 15 January 2026"
  const heading = page.locator('.planner-header h1');
  await expect(heading).toContainText('January 2026');
});

test('daily planner has navigation buttons', async ({ page }) => {
  await page.goto('/planner/daily/2026-01-15');
  await expect(page.locator('daily-planner-screen')).toBeVisible();

  // Buttons are created by createPlannerHeader with span text labels
  await expect(page.locator('.planner-header-button', { hasText: 'Previous Day' })).toBeVisible();
  await expect(page.locator('.planner-header-button', { hasText: 'Today' })).toBeVisible();
  await expect(page.locator('.planner-header-button', { hasText: 'Next Day' })).toBeVisible();
});

test('"Previous Day" button navigates to the previous date', async ({ page }) => {
  await page.goto('/planner/daily/2026-01-15');
  await expect(page.locator('daily-planner-screen')).toBeVisible();

  await page.locator('.planner-header-button', { hasText: 'Previous Day' }).click();
  await expect(page).toHaveURL('/planner/daily/2026-01-14');
});

test('"Next Day" button navigates to the next date', async ({ page }) => {
  await page.goto('/planner/daily/2026-01-15');
  await expect(page.locator('daily-planner-screen')).toBeVisible();

  await page.locator('.planner-header-button', { hasText: 'Next Day' }).click();
  await expect(page).toHaveURL('/planner/daily/2026-01-16');
});

test('"Today" button navigates to today', async ({ page }) => {
  await page.goto('/planner/daily/2026-01-15');
  await expect(page.locator('daily-planner-screen')).toBeVisible();

  await page.locator('.planner-header-button', { hasText: 'Today' }).click();
  await expect(page).toHaveURL(/\/planner\/daily\/\d{4}-\d{2}-\d{2}/);

  const url = page.url();
  const today = new Date().toISOString().slice(0, 10);
  expect(url).toContain(today);
});

test('daily planner shows comment composer', async ({ page }) => {
  await page.goto('/planner/daily/2026-01-15');
  await expect(page.locator('daily-planner-screen')).toBeVisible();

  await expect(page.locator(COMMENTS.view)).toBeVisible();
  await expect(page.locator(COMMENTS.draftTextarea)).toBeVisible();
  await expect(page.locator(COMMENTS.postButton)).toBeVisible();
});

test('weekly planner renders', async ({ page }) => {
  await page.goto('/planner/weekly/2026-W03');
  await expect(page.locator('weekly-planner-screen')).toBeVisible();
  await expect(page.locator('.planner-header h1')).toBeVisible();
});

test('monthly planner renders', async ({ page }) => {
  await page.goto('/planner/monthly/2026-01');
  await expect(page.locator('monthly-planner-screen')).toBeVisible();
  await expect(page.locator('.planner-header h1')).toBeVisible();
});

test('annual planner renders', async ({ page }) => {
  await page.goto('/planner/annual/2026');
  await expect(page.locator('annual-planner-screen')).toBeVisible();
  await expect(page.locator('.planner-header h1')).toBeVisible();
});

test('sidebar "Daily Planner" button opens today\'s planner', async ({ page }) => {
  await page.click(SIDEBAR.dailyPlanner);
  await expect(page.locator('daily-planner-screen')).toBeVisible({ timeout: 10_000 });

  const today = new Date().toISOString().slice(0, 10);
  await expect(page).toHaveURL(new RegExp(`/planner/daily/${today}`));
});

test('invalid daily planner date shows error card', async ({ page }) => {
  await page.goto('/planner/daily/not-a-date');
  await expect(page.locator('daily-planner-screen')).toBeVisible();
  await expect(page.locator('.planner-error-card')).toBeVisible();
  await expect(page.locator('.planner-error-card')).toContainText('Invalid day');
});
