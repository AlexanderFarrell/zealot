import { test, expect } from '@playwright/test';
import { generateSuffix } from '../helpers/unique';
import { LOGIN, REGISTER, APP_SHELL } from '../helpers/selectors';

// Helper: register a user via API then log in via the UI
async function registerAndLogin(
  page: Parameters<Parameters<typeof test>[1]>[0]['page'],
  request: Parameters<Parameters<typeof test>[1]>[0]['request'],
  username: string,
  password: string,
) {
  await request.post('/api/auth/register', {
    data: {
      username,
      password,
      confirm: password,
      email: `${username}@example.com`,
      given_name: 'T',
      surname: 'E',
    },
  });
  await page.goto('/');
  await page.fill(LOGIN.usernameInput, username);
  await page.fill(LOGIN.passwordInput, password);
  await page.click(LOGIN.submitButton);
  await expect(page.locator(APP_SHELL.webClient)).toBeVisible({ timeout: 15_000 });
}

test('shows login form when unauthenticated', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator(LOGIN.screen)).toBeVisible();
  await expect(page.locator(LOGIN.usernameInput)).toBeVisible();
  await expect(page.locator(LOGIN.passwordInput)).toBeVisible();
});

test('login with wrong password shows error', async ({ page, request }) => {
  const suffix = generateSuffix();
  const username = `auth_err_${suffix}`;
  await request.post('/api/auth/register', {
    data: {
      username,
      password: 'correct-password',
      confirm: 'correct-password',
      email: `${username}@example.com`,
      given_name: 'T',
      surname: 'E',
    },
  });

  await page.goto('/');
  await page.fill(LOGIN.usernameInput, username);
  await page.fill(LOGIN.passwordInput, 'wrong-password');
  await page.click(LOGIN.submitButton);

  await expect(page.locator(LOGIN.errorMessage)).not.toBeEmpty();
  await expect(page.locator(LOGIN.screen)).toBeVisible();
});

test('successful login shows main app', async ({ page, request }) => {
  const suffix = generateSuffix();
  await registerAndLogin(page, request, `auth_ok_${suffix}`, 'TestPass123!');

  await expect(page.locator(APP_SHELL.webClient)).toBeVisible();
  await expect(page.locator(LOGIN.screen)).not.toBeVisible();
});

test('session persists after page reload', async ({ page, request }) => {
  const suffix = generateSuffix();
  await registerAndLogin(page, request, `auth_persist_${suffix}`, 'TestPass123!');

  await page.reload();
  await expect(page.locator(APP_SHELL.webClient)).toBeVisible({ timeout: 10_000 });
  await expect(page.locator(LOGIN.screen)).not.toBeVisible();
});

test('register link navigates to register form', async ({ page }) => {
  await page.goto('/');
  await page.click(LOGIN.registerLink);

  await expect(page.locator(REGISTER.screen)).toBeVisible();
  await expect(page.locator(REGISTER.usernameInput)).toBeVisible();
});

test('register via UI with valid data succeeds', async ({ page }) => {
  const suffix = generateSuffix();
  const username = `auth_reg_${suffix}`;

  await page.goto('/');
  await page.click(LOGIN.registerLink);
  await expect(page.locator(REGISTER.screen)).toBeVisible();

  await page.fill(REGISTER.usernameInput, username);
  await page.fill(REGISTER.passwordInput, 'TestPass123!');
  await page.fill(REGISTER.confirmInput, 'TestPass123!');
  await page.fill(REGISTER.emailInput, `${username}@example.com`);
  await page.fill(REGISTER.givenNameInput, 'Test');
  await page.fill(REGISTER.surnameInput, 'User');
  await page.click(REGISTER.submitButton);

  await expect(page.locator(APP_SHELL.webClient)).toBeVisible({ timeout: 15_000 });
});

test('register with mismatched passwords shows error', async ({ page }) => {
  const suffix = generateSuffix();
  const username = `auth_mismatch_${suffix}`;

  await page.goto('/');
  await page.click(LOGIN.registerLink);

  await page.fill(REGISTER.usernameInput, username);
  await page.fill(REGISTER.passwordInput, 'TestPass123!');
  await page.fill(REGISTER.confirmInput, 'DifferentPass456!');
  await page.fill(REGISTER.emailInput, `${username}@example.com`);
  await page.fill(REGISTER.givenNameInput, 'Test');
  await page.fill(REGISTER.surnameInput, 'User');
  await page.click(REGISTER.submitButton);

  await expect(page.locator(REGISTER.errorMessage)).not.toBeEmpty();
  await expect(page.locator(REGISTER.screen)).toBeVisible();
});

test('login link on register form goes back to login', async ({ page }) => {
  await page.goto('/');
  await page.click(LOGIN.registerLink);
  await expect(page.locator(REGISTER.screen)).toBeVisible();

  await page.click(REGISTER.loginLink);
  await expect(page.locator(LOGIN.screen)).toBeVisible();
});
