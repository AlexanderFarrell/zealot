import { test as base, expect } from '@playwright/test';
import { generateSuffix } from '../helpers/unique';

/**
 * For specs that change account-wide settings (Z269 preferences). The suite shares one user, so a
 * setting changed there would leak into every parallel test. Here each test gets its own account,
 * registered and signed in before the test body runs.
 */
export const test = base.extend<{ createOwnItem: (title: string, content?: string) => Promise<number> }>({
  // The shared session from the project config is replaced by an empty one.
  storageState: async ({}, use) => { await use({ cookies: [], origins: [] }); },

  page: async ({ page }, use) => {
    const username = `own_${generateSuffix()}`;
    const password = 'TestPass123!';
    const registered = await page.request.post('/api/auth/register', {
      data: { username, password, confirm: password, email: `${username}@example.com`, given_name: 'Own', surname: 'Account' },
    });
    // Registering also signs the browser context in.
    expect(registered.ok(), await registered.text()).toBe(true);
    // The Home item is what the app opens at '/'.
    await createItem(page, 'Home', 'Home');
    await use(page);
  },

  createOwnItem: async ({ page }, use) => {
    await use((title, content = '') => createItem(page, title, content));
  },
});

async function createItem(page: import('@playwright/test').Page, title: string, content: string): Promise<number> {
  const csrf = (await page.context().storageState()).cookies.find((c) => c.name === 'csfr_')?.value;
  const res = await page.request.post('/api/item', { data: { title, content }, headers: csrf ? { 'X-Csrf-Token': csrf } : undefined });
  expect(res.ok(), await res.text()).toBe(true);
  return (await res.json() as { item_id: number }).item_id;
}

export { expect };
