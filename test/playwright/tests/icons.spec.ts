import type { APIRequestContext, Page } from '@playwright/test';
import { test, expect } from '../fixtures';
import { expandAttributes } from '../helpers/attributes';
import { uniqueTitle } from '../helpers/unique';
import { csrfHeaders } from '../helpers/csrf';
import { ADD_ITEM, APP_SHELL, COMMENTS, ITEM, SIDEBAR } from '../helpers/selectors';

const CTRL_N = 'Control+n';

// Search results and sidebar cards load over the API; with the suite fully parallel
// they can take longer than the default 5 s. They pass serially in well under 1 s.
const SEARCH_TIMEOUT = 15_000;

const expectApiOk = async (response: { ok(): boolean; status(): number; text(): Promise<string> }) => {
  if (response.ok()) return;
  throw new Error(`API request failed: ${response.status()} ${await response.text()}`);
};

const patchItemAttributes = async (
  request: APIRequestContext,
  itemId: number,
  attributes: Record<string, unknown>,
): Promise<void> => {
  const response = await request.patch(`/api/item/${itemId}/attr`, { data: attributes, headers: await csrfHeaders(request) });
  await expectApiOk(response);
};

// A type can only require attribute kinds that exist; the server silently drops
// unknown keys from `required_attributes`, so make sure "Icon" is a real kind.
// Creating a duplicate key is a 500, so look first.
const ensureAttributeKind = async (request: APIRequestContext, key: string): Promise<void> => {
  const existing = await request.get('/api/attribute');
  await expectApiOk(existing);
  const kinds = await existing.json() as Array<{ key: string }>;
  if (kinds.some(kind => kind.key === key)) return;
  const response = await request.post('/api/attribute', {
    data: { key, description: `${key} test kind`, base_type: 'text', config: {} },
    headers: await csrfHeaders(request),
  });
  await expectApiOk(response);
};

const openItemScreen = async (page: Page, title: string, { attributes = false } = {}): Promise<void> => {
  await page.goto(`/item/${encodeURIComponent(title)}`);
  await expect(page.locator(ITEM.screen)).toBeVisible();
  if (attributes) await expandAttributes(page);
};

test.beforeEach(async ({ page }) => {
  await page.goto('/');
  await expect(page.locator(APP_SHELL.webClient)).toBeVisible({ timeout: 10_000 });
});

test('icons render in item header, nav tree, search tool, and child cards', async ({ page, createItem, request }) => {
  const parentTitle = uniqueTitle('IconParent');
  const childTitle = uniqueTitle('IconChild');
  const searchTitle = uniqueTitle('IconSearch');

  const parentId = await createItem(parentTitle);
  const childId = await createItem(childTitle);
  const searchId = await createItem(searchTitle);

  // The navigation tree lists only Root items.
  await patchItemAttributes(request, parentId, { Icon: ':firefox:', Root: true });
  await patchItemAttributes(request, childId, {
    Icon: ':alarm-clock-check:',
    Parent: [parentId],
  });
  await patchItemAttributes(request, searchId, { Icon: ':access-point:' });

  await openItemScreen(page, parentTitle);

  const headerFirefoxIcon = page.locator('item-screen .item-title-row .zealot-icon[data-icon-ref="firefox"] svg');
  await expect(headerFirefoxIcon).toBeVisible();
  await expect(headerFirefoxIcon).toHaveAttribute('data-native-color', 'true');
  await expect(headerFirefoxIcon).toHaveAttribute('data-icon-source', 'simple');

  const navTitle = page.locator('.nav-tree-title', { hasText: parentTitle }).first();
  await expect(navTitle).toBeVisible();
  await expect(navTitle.locator('.zealot-icon[data-icon-ref="firefox"] svg')).toBeVisible();

  const childCard = page.locator('right-sidebar .item-card', { hasText: childTitle }).first();
  await expect(childCard).toBeVisible({ timeout: SEARCH_TIMEOUT });
  const childAlarmIcon = childCard.locator('.item-card__title .zealot-icon[data-icon-ref="alarm-clock-check"] svg');
  await expect(childAlarmIcon).toBeVisible();
  await expect(childAlarmIcon).toHaveAttribute('data-icon-source', 'lucide');
  await expect(childAlarmIcon).not.toHaveAttribute('data-native-color', 'true');

  await page.click(SIDEBAR.search);
  const searchInput = page.locator('search-tool-view input[type="search"]');
  await expect(searchInput).toBeVisible();
  await searchInput.fill(searchTitle);

  const searchResult = page.locator('.search-tool-result', { hasText: searchTitle }).first();
  await expect(searchResult).toBeVisible({ timeout: SEARCH_TIMEOUT });
  const accessPointIcon = searchResult.locator('.search-tool-result-title .zealot-icon[data-icon-ref="access-point"] svg');
  await expect(accessPointIcon).toBeVisible();
  await expect(accessPointIcon).toHaveAttribute('data-icon-source', 'mdi');
  await expect(accessPointIcon).not.toHaveAttribute('data-native-color', 'true');
});

test('item search inputs keep plain text values and show an icon preview', async ({ page, createItem, request }) => {
  const title = uniqueTitle('IconPreview');
  const itemId = await createItem(title);
  await patchItemAttributes(request, itemId, { Icon: ':firefox:' });

  await page.keyboard.press(CTRL_N);
  await expect(page.locator(ADD_ITEM.modal)).toBeVisible();

  const searchInput = page.locator('add-item-modal .item-search-inline input[type="search"]');
  await searchInput.fill(title);

  const resultRow = page.locator('add-item-modal .item-search-inline-row', { hasText: title }).first();
  await expect(resultRow).toBeVisible({ timeout: SEARCH_TIMEOUT });
  await expect(resultRow.locator('.zealot-icon[data-icon-ref="firefox"] svg')).toBeVisible();

  await resultRow.click();

  // Parent Item is a chips input: the selection becomes a chip carrying the icon.
  await expect(searchInput).toHaveValue('');
  const chip = page.locator('add-item-modal .chip_item', { hasText: title });
  await expect(chip).toBeVisible();
  await expect(chip.locator('.zealot-icon[data-icon-ref="firefox"] svg')).toBeVisible();
});

test('item search previews still render emoji item icons', async ({ page, createItem, request }) => {
  const title = uniqueTitle('EmojiPreview');
  const itemId = await createItem(title);
  await patchItemAttributes(request, itemId, { Icon: '🔥' });

  await page.keyboard.press(CTRL_N);
  await expect(page.locator(ADD_ITEM.modal)).toBeVisible();

  const searchInput = page.locator('add-item-modal .item-search-inline input[type="search"]');
  await searchInput.fill(title);

  const resultRow = page.locator('add-item-modal .item-search-inline-row', { hasText: title }).first();
  await expect(resultRow).toBeVisible({ timeout: SEARCH_TIMEOUT });
  await expect(resultRow.locator('.item-title-inline-icon--text')).toHaveText('🔥');

  await resultRow.click();

  const chip = page.locator('add-item-modal .chip_item', { hasText: title });
  await expect(chip).toBeVisible();
  await expect(chip.locator('.item-title-inline-icon--text')).toHaveText('🔥');
});

test('item attribute Icon fields use the shortcode picker and save icon shortcodes', async ({ page, createItem, request }) => {
  const title = uniqueTitle('AttrIcon');
  const itemId = await createItem(title);
  await patchItemAttributes(request, itemId, { Icon: ':folder:' });

  await openItemScreen(page, title, { attributes: true });

  const iconRow = page.locator('.attribute').first();
  await expect(iconRow.locator('input[type="text"]').first()).toHaveValue('Icon');
  const iconInput = iconRow.locator('span[name="value_view"] input[type="text"]');

  await expect(iconInput).toHaveValue(':folder:');
  await iconInput.fill('');
  await iconInput.pressSequentially(':clock-che');

  const alarmOption = page.locator('.zealotscript-emoji-picker .zealotscript-emoji-option', { hasText: 'alarm-clock-check' }).first();
  await expect(alarmOption).toBeVisible();
  await expect(alarmOption.locator('.zealot-icon[data-icon-ref="alarm-clock-check"] svg')).toBeVisible();

  await alarmOption.click();
  await expect(iconInput).toHaveValue(':alarm-clock-check:');

  await page.locator(ITEM.title).click();
  await expect(page.locator('item-screen .item-title-row .zealot-icon[data-icon-ref="alarm-clock-check"] svg')).toBeVisible();
});

test('adding an Icon attribute can insert emoji from the shortcode picker', async ({ page, createItem }) => {
  const title = uniqueTitle('AttrEmoji');
  await createItem(title);

  await openItemScreen(page, title, { attributes: true });

  const addRow = page.locator('.attribute', {
    has: page.locator('input[placeholder="New key"]'),
  }).first();
  const keyInput = addRow.locator('input[placeholder="New key"]');

  await keyInput.fill('Icon');

  const valueInput = addRow.locator('span[name="value_view"] input[type="text"]');
  await valueInput.click();
  await page.keyboard.type(':fir');

  const fireOption = page.locator('.zealotscript-emoji-picker .zealotscript-emoji-option', { hasText: 'fire' }).first();
  await expect(fireOption).toBeVisible();
  await expect(fireOption.locator('.zealotscript-shortcode-option-glyph')).toHaveText('🔥');

  await fireOption.click();
  await expect(valueInput).toHaveValue('🔥');

  await addRow.locator('button[type="button"]').click();
  await expect(page.locator('item-screen .item-title-row .item-title-inline-icon--text')).toHaveText('🔥');
});

test('shared attribute Icon inputs outside the item screen also use the shortcode picker', async ({ page, request }) => {
  const typeName = uniqueTitle('IconType');
  await ensureAttributeKind(request, 'Icon');
  const response = await request.post('/api/item_type', {
    data: { name: typeName, description: '', required_attributes: ['Icon'] },
    headers: await csrfHeaders(request),
  });
  await expectApiOk(response);

  await page.goto(`/types/${encodeURIComponent(typeName)}`);
  await expect(page.locator('type-screen')).toBeVisible();

  const createRow = page.locator('.item-table-create-row');
  // Column order is Title, Types (read-only text here), then the attributes.
  const iconInput = createRow.locator('input[type="text"]:not([data-item-table-create-title])').first();
  await expect(iconInput).toBeVisible();

  await iconInput.click();
  await page.keyboard.type(':ccess-po');

  const accessPointOption = page.locator('.zealotscript-emoji-picker .zealotscript-emoji-option', { hasText: 'access-point' }).first();
  await expect(accessPointOption).toBeVisible();
  await expect(accessPointOption.locator('.zealot-icon[data-icon-ref="access-point"] svg')).toBeVisible();

  await accessPointOption.click();
  await expect(iconInput).toHaveValue(':access-point:');
});

test('the shortcode picker includes icon entries and inserts icon refs', async ({ page, createItem }) => {
  // Z234: the picker opens below the fold in the bottom-docked composer, so its options cannot be clicked.
  test.fail(true, 'Z234 shortcode picker is not kept inside the viewport');
  const title = uniqueTitle('IconPicker');
  await createItem(title);

  await openItemScreen(page, title);

  const composerEditor = page.locator('comments-view zealotscript-editor[data-comments-draft="true"] .ProseMirror');
  // The composer is collapsed until the toggle is clicked.
  await page.locator(COMMENTS.composerToggle).click();
  await expect(composerEditor).toBeVisible();
  // click() would hit the sticky toolbar when the editor is scrolled into view.
  await composerEditor.focus();
  await page.keyboard.type(':fox');

  const firefoxOption = page.locator('.zealotscript-emoji-picker .zealotscript-emoji-option', { hasText: 'firefox' }).first();
  await expect(firefoxOption).toBeVisible();
  await expect(firefoxOption.locator('.zealot-icon[data-icon-ref="firefox"] svg')).toBeVisible();

  await firefoxOption.click({ timeout: 3_000 });
  await expect(composerEditor.locator('.zealot-icon[data-icon-ref="firefox"] svg')).toBeVisible();
});

test('the shortcode picker hides shadowed collisions and keeps winning source precedence', async ({ page, createItem }) => {
  // Z234: the picker opens below the fold in the bottom-docked composer, so its options cannot be clicked.
  test.fail(true, 'Z234 shortcode picker is not kept inside the viewport');
  const title = uniqueTitle('TargetPicker');
  await createItem(title);

  await openItemScreen(page, title, { attributes: true });

  const composerEditor = page.locator('comments-view zealotscript-editor[data-comments-draft="true"] .ProseMirror');
  // The composer is collapsed until the toggle is clicked.
  await page.locator(COMMENTS.composerToggle).click();
  await expect(composerEditor).toBeVisible();
  // click() would hit the sticky toolbar when the editor is scrolled into view.
  await composerEditor.focus();
  await page.keyboard.type(':target');

  const targetLabels = page.locator('.zealotscript-emoji-picker .zealotscript-shortcode-option-label').filter({ hasText: /^target$/ });
  await expect(targetLabels).toHaveCount(1);

  // `has` is evaluated relative to each option, so it must not be page-rooted.
  const targetOption = page
    .locator('.zealotscript-emoji-picker .zealotscript-emoji-option')
    .filter({ has: page.locator('.zealotscript-shortcode-option-label', { hasText: /^target$/ }) })
    .first();
  await expect(targetOption.locator('.zealot-icon[data-icon-ref="target"] svg')).toHaveAttribute('data-icon-source', 'simple');

  await targetOption.click({ timeout: 3_000 });
  await expect(composerEditor.locator('.zealot-icon[data-icon-ref="target"] svg')).toHaveAttribute('data-icon-source', 'simple');
});

test('icon shortcodes render in the editor and persist as shortcodes in rendered comments', async ({ page, createItem, request }) => {
  const title = uniqueTitle('IconComment');
  const itemId = await createItem(title);

  await openItemScreen(page, title);

  const composerEditor = page.locator('comments-view zealotscript-editor[data-comments-draft="true"] .ProseMirror');
  // The composer is collapsed until the toggle is clicked.
  await page.locator(COMMENTS.composerToggle).click();
  await expect(composerEditor).toBeVisible();
  // click() would hit the sticky toolbar when the editor is scrolled into view.
  await composerEditor.focus();
  await page.keyboard.type('Known :firefox: :alarm-clock-check: :access-point: Emoji :apple: Unknown :notrealicon:');

  await expect(composerEditor.locator('.zealot-icon[data-icon-ref="firefox"] svg')).toBeVisible();
  await expect(composerEditor.locator('.zealot-icon[data-icon-ref="alarm-clock-check"] svg')).toBeVisible();
  await expect(composerEditor.locator('.zealot-icon[data-icon-ref="access-point"] svg')).toBeVisible();
  await expect(composerEditor).toContainText('🍎');

  await page.waitForTimeout(700);
  await page.locator('.comments-view-composer button[type="submit"]').click();

  const commentCard = page.locator('.comments-view-card').first();
  await expect(commentCard).toBeVisible({ timeout: SEARCH_TIMEOUT });
  await expect(commentCard.locator('.comments-view-content .zealot-icon[data-icon-ref="firefox"] svg')).toBeVisible();
  await expect(commentCard.locator('.comments-view-content .zealot-icon[data-icon-ref="alarm-clock-check"] svg')).toBeVisible();
  await expect(commentCard.locator('.comments-view-content .zealot-icon[data-icon-ref="access-point"] svg')).toBeVisible();
  await expect(commentCard.locator('.comments-view-content')).toContainText('🍎');
  await expect(commentCard.locator('.comments-view-content')).toContainText(':notrealicon:');

  const response = await request.get(`/api/comment/item/${itemId}`);
  await expectApiOk(response);
  const comments = await response.json() as Array<{ comment_id: number; content: string }>;
  const saved = comments.find((comment) => comment.content.includes(':firefox:'));
  expect(saved).toBeTruthy();
  expect(saved?.content).toContain(':alarm-clock-check:');
  expect(saved?.content).toContain(':access-point:');
  expect(saved?.content).toContain('🍎');
  expect(saved?.content).toContain(':notrealicon:');
  expect(saved?.content).not.toContain(':apple:');
  expect(saved?.content).not.toContain('<svg');

  await page.reload();
  const reloadedComment = page.locator('.comments-view-card').first();
  await expect(reloadedComment.locator('.comments-view-content .zealot-icon[data-icon-ref="firefox"] svg')).toBeVisible();
  await expect(reloadedComment.locator('.comments-view-content .zealot-icon[data-icon-ref="alarm-clock-check"] svg')).toBeVisible();
  await expect(reloadedComment.locator('.comments-view-content .zealot-icon[data-icon-ref="access-point"] svg')).toBeVisible();
  await expect(reloadedComment.locator('.comments-view-content')).toContainText('🍎');
  await expect(reloadedComment.locator('.comments-view-content')).toContainText(':notrealicon:');
});
