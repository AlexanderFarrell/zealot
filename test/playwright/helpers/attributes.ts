import type { Page } from '@playwright/test';

/** Opens the item screen's attribute table, which starts as a summary line (Z240, Z253). */
export async function expandAttributes(page: Page): Promise<void> {
  const table = page.locator('item-screen .attribute-editor');
  if (!(await table.isVisible())) {
    await page.locator('item-screen .attribute-summary').click();
  }
  await table.waitFor({ state: 'visible' });
}
