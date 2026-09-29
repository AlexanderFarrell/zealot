import { test, expect } from '../fixtures';
import { uniqueTitle } from '../helpers/unique';
import { CONFIRM, ITEM } from '../helpers/selectors';

test('Statistic item supports quick entry, pagination, editing, deletion, and formatting', async ({ page, request }) => {
  const title = uniqueTitle('Statistic');
  const csrf = (await request.storageState()).cookies.find(cookie => cookie.name === 'csfr_')?.value;
  const headers = csrf ? { 'X-Csrf-Token': csrf } : undefined;
  const created = await request.post('/api/item', {
    data: {
      title,
      content: '',
      types: ['Statistic'],
      attributes: {
        'Value Kind': 'Number',
        Unit: 'kg',
        'Daily Aggregation': 'Average',
      },
    },
    headers,
  });
  expect(created.ok(), await created.text()).toBeTruthy();
  const item = await created.json() as { item_id: number };

  for (let index = 0; index < 21; index += 1) {
    const response = await request.post(`/api/statistic/${item.item_id}/entries`, {
      data: {
        value: 70 + index,
        occurred_at: `2026-08-${String(index + 1).padStart(2, '0')}T08:00:00Z`,
      },
      headers,
    });
    expect(response.ok(), await response.text()).toBeTruthy();
  }

  await page.goto(`/item/${encodeURIComponent(title)}`);
  await expect(page.locator(ITEM.screen)).toBeVisible({ timeout: 10_000 });
  const statistics = page.locator('statistics-view');
  await expect(statistics).toBeVisible();
  await expect(statistics.locator('.statistics-view-entry')).toHaveCount(20);
  await expect(statistics.getByText('90 kg', { exact: true })).toBeVisible();
  await expect(statistics.getByText('Latest delta: +1 kg', { exact: true })).toBeVisible();

  await statistics.getByRole('button', { name: 'Load more' }).click();
  await expect(statistics.locator('.statistics-view-entry')).toHaveCount(21);

  await statistics.locator('.statistics-view-entry').first().getByRole('button', { name: 'Edit' }).click();
  const editor = statistics.locator('.statistics-view-editor');
  await editor.locator('input[type="number"]').fill('91.5');
  await editor.getByRole('button', { name: 'Save' }).click();
  await expect(statistics.getByText('91.5 kg', { exact: true })).toBeVisible();

  const composer = statistics.locator('.statistics-view-composer');
  await composer.locator('input[type="number"]').fill('92');
  await composer.getByRole('button', { name: 'Record' }).click();
  await expect(statistics.getByText('92 kg', { exact: true })).toBeVisible();

  await statistics.locator('.statistics-view-entry').first().getByRole('button', { name: 'Delete' }).click();
  await expect(page.locator(CONFIRM.dialog)).toBeVisible();
  await page.locator(CONFIRM.confirmButton).click();
  await expect(statistics.getByText('92 kg', { exact: true })).not.toBeVisible();
});
