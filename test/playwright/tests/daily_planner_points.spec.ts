import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures/own_account';

// Z277: the daily planner groups by a dropdown, scores by configured attributes,
// moves configured types to the header and carries slips over. Each test has
// its own account, so nothing else is dated today and settings don't leak.

function isoDaysAgo(days: number): string {
  const d = new Date();
  d.setDate(d.getDate() - days);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
}

async function csrf(page: Page): Promise<Record<string, string>> {
  const token = (await page.context().storageState()).cookies.find((c) => c.name === 'csfr_')?.value;
  return token ? { 'X-Csrf-Token': token } : {};
}

async function add(page: Page, title: string, attributes: Record<string, unknown>, types: string[] = []): Promise<number> {
  for (const name of types) {
    // Creating a type that already exists fails harmlessly.
    await page.request.post('/api/item_type', { data: { name, description: name, required_attributes: [] }, headers: await csrf(page) });
  }
  const res = await page.request.post('/api/item', {
    data: { title, content: '', attributes, ...(types.length > 0 ? { types } : {}) },
    headers: await csrf(page),
  });
  expect(res.ok(), await res.text()).toBe(true);
  return (await res.json() as { item_id: number }).item_id;
}

/** Creates the Executor dropdown with this value order (an existing one is left as is). */
async function ensureExecutorKind(page: Page): Promise<void> {
  await page.request.post('/api/attribute', {
    data: { key: 'Executor', description: 'Who does it', base_type: 'dropdown', config: { values: ['Alexander', 'Mixed', 'LLM', 'External'] } },
    headers: await csrf(page),
  });
}

async function saveSettings(page: Page, settings: Record<string, unknown>): Promise<void> {
  const res = await page.request.patch('/api/account/settings', { data: settings, headers: await csrf(page) });
  expect(res.ok(), await res.text()).toBe(true);
}

function settingsSaved(page: Page): Promise<unknown> {
  return page.waitForResponse((r) => r.url().endsWith('/api/account/settings') && r.request().method() === 'PATCH');
}

const today = isoDaysAgo(0);

test('configured planner groups by Executor, scores and carries slips over (Z277)', async ({ page }) => {
  await ensureExecutorKind(page);
  await saveSettings(page, {
    planner_group_by: 'Executor',
    planner_points_from: ['Priority', 'AP'],
    planner_header_types: ['Log'],
    planner_collapsed_groups: {},
    planner_card_fields: ['Status', 'Executor', 'Priority', 'AP'],
  });
  await add(page, 'Mine low', { Date: today, Priority: 2, Status: 'To Do', Executor: 'Alexander' });
  await add(page, 'Mine high done', { Date: today, Priority: 8, AP: 2, Status: 'Complete', Executor: 'Alexander' });
  await add(page, 'LLM work', { Date: today, Priority: 5, Status: 'Working', Executor: 'LLM' });
  await add(page, 'Nobody', { Date: today, Priority: 1, Status: 'To Do' });
  await add(page, `Log ${today}`, { Date: today }, ['Log']);
  await add(page, 'Slipped mine', { Date: isoDaysAgo(1), Priority: 3, Status: 'To Do', Executor: 'Alexander' });
  await add(page, 'Slipped LLM', { Date: isoDaysAgo(2), Priority: 1, Status: 'To Do', Executor: 'LLM' });
  await add(page, 'Held', { Date: isoDaysAgo(1), Priority: 3, Status: 'Hold' });
  await add(page, 'Too old', { Date: isoDaysAgo(15), Priority: 3, Status: 'To Do' });
  // Notes have no Status: they sit in Other, never carry over and never score.
  await add(page, 'A note', { Date: today, Priority: 9 });
  await add(page, 'Old note', { Date: isoDaysAgo(1), Priority: 9 });

  await page.goto(`/planner/daily/${today}`);
  const screen = page.locator('daily-planner-screen');
  const score = screen.locator('.planner-day-score');

  // Earned 1,600; planned 200 + 1,600 + 500 + 100 today, plus 300 + 100 carried = 2,800.
  await expect(score).toContainText('1,600');
  await expect(score).toContainText('2,800 pts');
  await expect(screen.locator('.planner-day-log')).toHaveText(`Log ${today}`);
  await expect(screen.locator('.item-card', { hasText: `Log ${today}` })).toHaveCount(0);

  const items = screen.locator('.planner-section').filter({ has: page.locator('h2', { hasText: /^Items$/ }) });
  await expect(items.locator('.item-card-group__toggle')).toHaveText(['Alexander (2)', 'LLM (1)', 'No Executor (1)', 'Other (1)']);
  await expect(items.locator('.item-card', { hasText: 'A note' }).locator('.item-card__chip')).toHaveCount(0);
  const mineTitles = items.locator('.item-card-group').first().locator('.item-card__title');
  await expect(mineTitles).toHaveText(['Mine high done', 'Mine low']);
  await expect(items.locator('.item-card', { hasText: 'Mine high done' }).locator('.item-card__chip--points')).toHaveText('1,600 pts');

  // Carried Over starts collapsed with its count; one tap opens it, grouped the same way.
  const carried = screen.locator('.planner-section--carried');
  const carriedToggle = carried.locator('.collapsible-toggle');
  await expect(carriedToggle).toHaveText('Carried Over (2)');
  await expect(carried.locator('.item-card').first()).toBeHidden();
  await carriedToggle.click();
  await expect(carried.locator('.item-card-group__toggle')).toHaveText(['Alexander (1)', 'LLM (1)']);

  // Collapsing LLM hides it in both lists and is saved to the account.
  const saved = settingsSaved(page);
  await items.locator('.item-card-group__toggle', { hasText: 'LLM' }).click();
  await expect(items.locator('.item-card', { hasText: 'LLM work' })).toBeHidden();
  await expect(carried.locator('.item-card', { hasText: 'Slipped LLM' })).toBeHidden();
  await saved;

  // AP 9 on "Mine low" (2 × 9 = 1,800) moves it above the 1,600 item and the score follows.
  const low = items.locator('.item-card', { hasText: 'Mine low' });
  await low.locator('.item-card__chip--edit[data-key="AP"]').click();
  await low.locator('input.item-card__chip-input').fill('9');
  await low.locator('input.item-card__chip-input').press('Enter');
  await expect(mineTitles).toHaveText(['Mine low', 'Mine high done']);
  await expect(score).toContainText('4,400 pts');

  // Move the slipped item onto today.
  await carried.locator('.item-card', { hasText: 'Slipped mine' }).locator('.item-card__action').click();
  await expect(mineTitles).toHaveText(['Mine low', 'Mine high done', 'Slipped mine']);
  await expect(carriedToggle).toHaveText('Carried Over (1)');

  // The edits and the collapsed group survive a reload.
  await page.reload();
  await expect(mineTitles).toHaveText(['Mine low', 'Mine high done', 'Slipped mine']);
  await expect(score).toContainText('4,400 pts');
  await expect(items.locator('.item-card-group__toggle', { hasText: 'LLM' })).toHaveAttribute('aria-expanded', 'false');
  await expect(items.locator('.item-card', { hasText: 'LLM work' })).toBeHidden();
});

test('a new account groups by Status with no points and no header types (Z277)', async ({ page }) => {
  await add(page, 'Done thing', { Date: today, Priority: 8, Status: 'Complete' });
  await add(page, 'Open thing', { Date: today, Priority: 2, Status: 'To Do' });
  await add(page, 'Busy thing', { Date: today, Priority: 2, Status: 'Working' });
  await add(page, `Log ${today}`, { Date: today, Status: 'To Do' }, ['Log']);

  await page.goto(`/planner/daily/${today}`);
  const screen = page.locator('daily-planner-screen');
  await expect(screen.locator('.item-card-group__toggle')).toHaveText(['Working (1)', 'To Do (2)', 'Complete (1)']);
  await expect(screen.locator('.item-card__chip')).toHaveCount(0);
  await expect(screen.locator('.planner-day-score')).toHaveCount(0);
  await expect(screen.locator('.item-card', { hasText: `Log ${today}` })).toBeVisible();
});

test('carried-over items show only on today (Z277)', async ({ page }) => {
  await add(page, 'Slipped yesterday', { Date: isoDaysAgo(1), Priority: 3, Status: 'To Do' });
  await page.goto(`/planner/daily/${isoDaysAgo(-1)}`);
  await expect(page.locator('daily-planner-screen .item-card-empty')).toBeVisible();
  await expect(page.locator('daily-planner-screen .planner-section--carried')).toHaveCount(0);
});

test('planner settings save grouping, points and header types to the account (Z277)', async ({ page }) => {
  await ensureExecutorKind(page);
  await page.request.post('/api/attribute', {
    data: { key: 'Effort', description: 'Size of the job', base_type: 'integer', config: {} },
    headers: await csrf(page),
  });
  await add(page, 'Some log', {}, ['Log']);
  await page.goto('/settings/planner');
  const screen = page.locator('planner-settings-screen');

  let saved = settingsSaved(page);
  await screen.getByLabel('Group items by').selectOption('Executor');
  await saved;
  const points = screen.locator('.preference-checklist', { hasText: 'Points from' });
  await expect(points.getByRole('checkbox', { name: 'Executor' })).toHaveCount(0);
  saved = settingsSaved(page);
  await points.getByRole('checkbox', { name: 'Effort' }).check();
  await saved;
  saved = settingsSaved(page);
  await screen.locator('.preference-checklist', { hasText: 'Show in the header' }).getByRole('checkbox', { name: 'Log' }).check();
  await saved;
  const cardFields = screen.locator('.preference-checklist', { hasText: 'Edit on cards' });
  await expect(cardFields.getByRole('checkbox', { name: 'Status' })).toBeChecked();
  saved = settingsSaved(page);
  await cardFields.getByRole('checkbox', { name: 'Executor' }).check();
  await saved;

  const account = await (await page.request.get('/api/auth/is_logged_in')).json() as { settings: Record<string, unknown> };
  expect(account.settings.planner_group_by).toBe('Executor');
  expect(account.settings.planner_points_from).toEqual(['Effort']);
  expect(account.settings.planner_header_types).toEqual(['Log']);
  expect(account.settings.planner_card_fields).toEqual(['Status', 'Executor']);
});

test('card fields edit inline and a heading drop sets the group value (Z277)', async ({ page }) => {
  await ensureExecutorKind(page);
  await saveSettings(page, {
    planner_group_by: 'Executor',
    planner_points_from: ['Priority'],
    planner_header_types: [],
    planner_collapsed_groups: {},
    planner_card_fields: ['Executor', 'Priority'],
  });
  const unassigned = await add(page, 'Unassigned', { Date: today, Priority: 2, Status: 'To Do' });
  const dragged = await add(page, 'Drag me', { Date: today, Priority: 1, Status: 'To Do' });
  await add(page, 'Mine', { Date: today, Priority: 5, Status: 'To Do', Executor: 'Alexander' });

  await page.goto(`/planner/daily/${today}`);
  const screen = page.locator('daily-planner-screen');
  const toggles = screen.locator('.item-card-group__toggle');
  await expect(toggles).toHaveText(['Alexander (1)', 'No Executor (2)']);
  // Status isn't in "Edit on cards", so no corner select; points still show.
  await expect(screen.locator('.item-card__status-select')).toHaveCount(0);
  await expect(screen.locator('.item-card', { hasText: 'Mine' }).locator('.item-card__chip--points')).toHaveText('500 pts');

  // The Executor chip is a pick list; choosing a value moves the card and saves.
  const card = screen.locator('.item-card', { hasText: 'Unassigned' });
  await expect(card.locator('.item-card__chip--edit[data-key="Executor"]')).toHaveText('Executor –');
  await card.locator('.item-card__chip--edit[data-key="Executor"]').click();
  await card.locator('select.item-card__chip-select').selectOption('External');
  await expect(toggles).toHaveText(['Alexander (1)', 'External (1)', 'No Executor (1)']);
  await expect.poll(async () => (await (await page.request.get(`/api/item/id/${unassigned}`)).json() as { attributes: Record<string, unknown> }).attributes['Executor']).toBe('External');

  // Dragging a card onto a heading sets that value; onto "No Executor" clears it.
  await screen.locator('.item-card', { hasText: 'Drag me' }).dragTo(screen.locator('.item-card-group__label', { hasText: 'Alexander' }));
  await expect(toggles).toHaveText(['Alexander (2)', 'External (1)']);
  await screen.locator('.item-card', { hasText: 'Mine' }).dragTo(screen.locator('.item-card-group__label', { hasText: 'External' }));
  await expect(toggles).toHaveText(['Alexander (1)', 'External (2)']);
  await page.reload();
  await expect(toggles).toHaveText(['Alexander (1)', 'External (2)']);
  await expect.poll(async () => (await (await page.request.get(`/api/item/id/${dragged}`)).json() as { attributes: Record<string, unknown> }).attributes['Executor']).toBe('Alexander');
});

test('a card dropped on "No <Attribute>" loses the value (Z277)', async ({ page }) => {
  await ensureExecutorKind(page);
  await saveSettings(page, { planner_group_by: 'Executor', planner_points_from: [], planner_header_types: [], planner_collapsed_groups: {} });
  const id = await add(page, 'Was mine', { Date: today, Status: 'To Do', Executor: 'Alexander' });
  await add(page, 'Nobody yet', { Date: today, Status: 'To Do' });
  await page.goto(`/planner/daily/${today}`);
  const screen = page.locator('daily-planner-screen');
  await screen.locator('.item-card', { hasText: 'Was mine' }).dragTo(screen.locator('.item-card-group__label', { hasText: 'No Executor' }));
  await expect(screen.locator('.item-card-group__toggle')).toHaveText(['No Executor (2)']);
  await expect.poll(async () => (await (await page.request.get(`/api/item/id/${id}`)).json() as { attributes: Record<string, unknown> }).attributes['Executor']).toBeUndefined();
});
