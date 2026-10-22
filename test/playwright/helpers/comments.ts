import type { Page } from '@playwright/test';
import { expect } from '@playwright/test';

/** The Comments section starts collapsed on items and planners; opens it if it is closed. */
export async function expandComments(page: Page): Promise<void> {
  const toggle = page.locator('.collapsible-toggle', { hasText: 'Comments' });
  await expect(toggle).toBeVisible();
  if ((await toggle.getAttribute('aria-expanded')) !== 'true') await toggle.click();
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
}
