import { defineConfig, devices } from '@playwright/test';

/**
 * Two ways to run:
 *
 * 1. Against the Docker stack (recommended, fresh database):
 *      ./test/playwright/run.sh
 *    or from the project root:
 *      npm run test:playwright
 *
 *    run.sh builds and starts the Docker Compose stack, waits for it to be
 *    healthy, then sets PLAYWRIGHT_BASE_URL=http://127.0.0.1:18081 and runs
 *    Playwright. The stack is torn down automatically on exit, giving a fresh
 *    SQLite database every time.
 *
 * 2. Against the local dev server (no Docker required):
 *      cd test/playwright && npm test
 *
 *    Playwright auto-starts the Vite dev server. You must have the Rust
 *    backend running first: cd apps/server && cargo run
 */

const BASE_URL = process.env.PLAYWRIGHT_BASE_URL ?? 'http://localhost:5173';
const usingDocker = !!process.env.PLAYWRIGHT_BASE_URL;

export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 2 : undefined,
  reporter: [['html', { outputFolder: 'playwright-report' }], ['list']],

  use: {
    baseURL: BASE_URL,
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },

  projects: [
    // Runs auth.setup.ts once to create .auth/user.json
    {
      name: 'setup',
      testMatch: /auth\.setup\.ts/,
    },

    // All spec files EXCEPT auth.spec.ts — use the shared logged-in session
    {
      name: 'chromium-authenticated',
      use: {
        ...devices['Desktop Chrome'],
        storageState: '.auth/user.json',
      },
      dependencies: ['setup'],
      testIgnore: /auth\.spec\.ts/,
    },

    // auth.spec.ts runs without a stored session (tests login/register flows)
    {
      name: 'chromium-auth',
      use: { ...devices['Desktop Chrome'] },
      testMatch: /auth\.spec\.ts/,
    },
  ],

  // Only start the Vite dev server when NOT targeting the Docker stack.
  // When PLAYWRIGHT_BASE_URL is set (i.e. run.sh), the stack is already running.
  webServer: usingDocker ? undefined : {
    command: 'npm run dev:web',
    url: 'http://localhost:5173',
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
    cwd: '../../',
  },
});
