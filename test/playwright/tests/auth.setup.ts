import { test as setup, expect } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';
import { generateSuffix } from '../helpers/unique';
import { LOGIN, APP_SHELL } from '../helpers/selectors';

const AUTH_FILE = path.join(__dirname, '../.auth/user.json');

/**
 * Creates a single shared user for the authenticated test suite and saves
 * the session cookie to .auth/user.json. All authenticated test projects
 * load this file via storageState in playwright.config.ts.
 */
setup('create authenticated session', async ({ page, request }) => {
  const suffix = generateSuffix();
  const username = `suite_${suffix}`;
  const password = 'TestPass123!';

  const res = await request.post('/api/auth/register', {
    data: {
      username,
      password,
      confirm: password,
      email: `${username}@example.com`,
      given_name: 'Suite',
      surname: 'User',
    },
  });
  expect(res.ok()).toBeTruthy();

  // Log in via the UI so the page context gets the session cookie
  await page.goto('/');
  await page.fill(LOGIN.usernameInput, username);
  await page.fill(LOGIN.passwordInput, password);
  await page.click(LOGIN.submitButton);
  await expect(page.locator(APP_SHELL.webClient)).toBeVisible({ timeout: 15_000 });

  // Create the Home item required by the home-page tests.
  // Use page.request (shares the browser context's cookies) not the standalone
  // request fixture (which has no session cookie at this point).
  const homeRes = await page.request.post('/api/item', {
    data: { title: 'Home', content: '' },
  });
  expect(homeRes.ok()).toBeTruthy();

  fs.mkdirSync(path.dirname(AUTH_FILE), { recursive: true });
  await page.context().storageState({ path: AUTH_FILE });
});
