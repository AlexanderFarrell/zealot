import { test, expect } from '../fixtures';
import { uniqueTitle, generateSuffix } from '../helpers/unique';
import { APP_SHELL } from '../helpers/selectors';

test('semantic attribute kinds render typed, linked, editable controls', async ({ page, request }) => {
  const suffix = generateSuffix();
  const csrf = (await request.storageState()).cookies.find((c) => c.name === 'csfr_')?.value;
  const headers = csrf ? { 'X-Csrf-Token': csrf } : undefined;

  const kinds: Array<[string, string, object]> = [
    [`Email ${suffix}`, 'email', {}],
    [`Phone ${suffix}`, 'telephone', {}],
    [`Site ${suffix}`, 'url', {}],
    [`File ${suffix}`, 'media', {}],
    [`Place ${suffix}`, 'place', {}],
    [`Tags ${suffix}`, 'list', { list_type: 'dropdown', values: ['red', 'green', 'blue'] }],
  ];
  for (const [key, base_type, config] of kinds) {
    const res = await request.post('/api/attribute', {
      data: { key, description: `${base_type} test`, base_type, config },
      headers,
    });
    expect(res.ok(), `create ${base_type}: ${await res.text()}`).toBeTruthy();
  }

  const title = uniqueTitle('SemanticAttrs');
  const res = await request.post('/api/item', {
    data: {
      title,
      content: '',
      attributes: {
        [`Email ${suffix}`]: 'me@example.com',
        [`Phone ${suffix}`]: '+1 555 123 4567',
        [`Site ${suffix}`]: 'https://example.com/page',
        [`File ${suffix}`]: '/api/media/paste/cat.png',
        [`Place ${suffix}`]: '51.5, -0.12',
        [`Tags ${suffix}`]: ['red', 'blue'],
      },
    },
    headers,
  });
  expect(res.ok(), await res.text()).toBeTruthy();

  await page.goto(`/item/${encodeURIComponent(title)}`);
  await expect(page.locator(APP_SHELL.webClient)).toBeVisible({ timeout: 10_000 });

  // Rows list attributes alphabetically; keys are input values, so address rows by position.
  const rows = page.locator('attribute-editor .attribute');
  await expect(rows.nth(5)).toBeVisible();
  const order = ['Email', 'File', 'Phone', 'Place', 'Site', 'Tags'];
  const row = (key: string) => rows.nth(order.indexOf(key.split(' ')[0]!));

  // URL: a live link beside the editable field, which follows edits.
  const site = row(`Site ${suffix}`);
  const siteLink = site.locator('a.semantic-attribute-link');
  await expect(siteLink).toHaveAttribute('href', 'https://example.com/page');
  await site.locator('input[type="url"]').fill('https://example.org/');
  await expect(siteLink).toHaveAttribute('href', 'https://example.org/');
  await site.locator('input[type="url"]').fill('javascript:alert(1)');
  await expect(siteLink).toBeHidden(); // Never linked.

  // Email and telephone use native inputs and mailto:/tel: links.
  await expect(row(`Email ${suffix}`).locator('a.semantic-attribute-link')).toHaveAttribute('href', 'mailto:me@example.com');
  await expect(row(`Phone ${suffix}`).locator('input[type="tel"]')).toHaveValue('+1 555 123 4567');
  await expect(row(`Phone ${suffix}`).locator('a.semantic-attribute-link')).toHaveAttribute('href', 'tel:+15551234567');

  // Media: a link plus an upload button.
  await expect(row(`File ${suffix}`).getByRole('button', { name: 'Upload' })).toBeVisible();
  await expect(row(`File ${suffix}`).locator('a.semantic-attribute-link')).toHaveAttribute('href', '/api/media/paste/cat.png');

  // Place: coordinates open a map.
  await expect(row(`Place ${suffix}`).locator('a.semantic-attribute-link')).toHaveAttribute('href', /openstreetmap\.org.*mlat=51\.5/);

  // Multi-select: checked boxes match the stored values and persist after a change.
  const tags = row(`Tags ${suffix}`);
  await expect(tags.locator('input[type="checkbox"]:checked')).toHaveCount(2);
  await tags.locator('input[type="checkbox"][value="green"]').check();
  await expect(async () => {
    const item = await (await request.get(`/api/item/title/${encodeURIComponent(title)}`)).text();
    expect(item).toContain('green');
  }).toPass({ timeout: 10_000 });
});
