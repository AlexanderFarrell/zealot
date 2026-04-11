import { test as base } from '@playwright/test';
import { generateSuffix } from '../helpers/unique';

export interface RegisteredUser {
  username: string;
  password: string;
  email: string;
}

export interface ApiFixture {
  /**
   * Register a new user via API. Call with a unique suffix to avoid collisions.
   * Returns the credentials (does NOT log the page in — use the login helpers for that).
   */
  registerUser: (suffix?: string) => Promise<RegisteredUser>;

  /**
   * Create an item via the API (requires the test to already be authenticated,
   * i.e. using the chromium-authenticated project). Returns the item_id.
   */
  createItem: (title: string, content?: string) => Promise<number>;
}

export const test = base.extend<ApiFixture>({
  registerUser: async ({ request }, use) => {
    const factory = async (suffix?: string): Promise<RegisteredUser> => {
      const s = suffix ?? generateSuffix();
      const username = `pw_${s}`;
      const password = 'TestPass123!';
      const email = `${username}@example.com`;
      const res = await request.post('/api/auth/register', {
        data: {
          username,
          password,
          confirm: password,
          email,
          given_name: 'PW',
          surname: 'Test',
        },
      });
      if (!res.ok()) {
        throw new Error(`registerUser failed: ${res.status()} ${await res.text()}`);
      }
      return { username, password, email };
    };
    await use(factory);
  },

  createItem: async ({ request }, use) => {
    const factory = async (title: string, content = ''): Promise<number> => {
      const res = await request.post('/api/item', {
        data: { title, content },
      });
      if (!res.ok()) {
        throw new Error(`createItem failed: ${res.status()} ${await res.text()}`);
      }
      const body = await res.json() as { item_id: number };
      return body.item_id;
    };
    await use(factory);
  },
});

export { expect } from '@playwright/test';
