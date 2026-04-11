import { test, expect } from '../fixtures';
import { uniqueTitle } from '../helpers/unique';
import { ITEM, COMMENTS, CONFIRM, APP_SHELL } from '../helpers/selectors';

test.beforeEach(async ({ page }) => {
  await page.goto('/');
  await expect(page.locator(APP_SHELL.webClient)).toBeVisible({ timeout: 10_000 });
});

async function navigateToItem(
  page: Parameters<Parameters<typeof test>[1]>[0]['page'],
  title: string,
) {
  await page.goto(`/item/${encodeURIComponent(title)}`);
  await expect(page.locator(ITEM.screen)).toBeVisible({ timeout: 10_000 });
}

test('comments section is visible on item screen', async ({ page, createItem }) => {
  const title = uniqueTitle('CommentsItem');
  await createItem(title);
  await navigateToItem(page, title);

  await expect(page.locator(COMMENTS.view)).toBeVisible();
});

test('fresh item shows "No comments yet."', async ({ page, createItem }) => {
  const title = uniqueTitle('NoComments');
  await createItem(title);
  await navigateToItem(page, title);

  await expect(page.locator(COMMENTS.empty)).toBeVisible();
  await expect(page.locator(COMMENTS.empty)).toContainText('No comments yet.');
});

test('comment composer is always visible', async ({ page, createItem }) => {
  const title = uniqueTitle('ComposerItem');
  await createItem(title);
  await navigateToItem(page, title);

  await expect(page.locator(COMMENTS.draftTextarea)).toBeVisible();
  await expect(page.locator(COMMENTS.postButton)).toBeVisible();
});

test('add a comment via the composer', async ({ page, createItem }) => {
  const title = uniqueTitle('AddComment');
  await createItem(title);
  await navigateToItem(page, title);

  const commentText = `Test comment ${Date.now()}`;
  await page.fill(COMMENTS.draftTextarea, commentText);
  await page.click(COMMENTS.postButton);

  await expect(page.locator(COMMENTS.card)).toBeVisible({ timeout: 10_000 });
  await expect(page.locator(COMMENTS.content)).toContainText(commentText);
});

test('draft textarea clears after posting', async ({ page, createItem }) => {
  const title = uniqueTitle('DraftClear');
  await createItem(title);
  await navigateToItem(page, title);

  await page.fill(COMMENTS.draftTextarea, 'Some draft text');
  await page.click(COMMENTS.postButton);

  await expect(page.locator(COMMENTS.card)).toBeVisible({ timeout: 10_000 });
  await expect(page.locator(COMMENTS.draftTextarea)).toHaveValue('');
});

test('posting blank comment shows error', async ({ page, createItem }) => {
  const title = uniqueTitle('BlankComment');
  await createItem(title);
  await navigateToItem(page, title);

  // Leave draft empty and click Post
  await page.click(COMMENTS.postButton);

  // comments_view.ts sets _draftError when content is empty
  await expect(page.locator(COMMENTS.draftError)).toBeVisible({ timeout: 5_000 });
});

test('edit a comment', async ({ page, createItem }) => {
  const title = uniqueTitle('EditComment');
  await createItem(title);
  await navigateToItem(page, title);

  // Post a comment first
  const original = `Original ${Date.now()}`;
  await page.fill(COMMENTS.draftTextarea, original);
  await page.click(COMMENTS.postButton);
  await expect(page.locator(COMMENTS.card)).toBeVisible({ timeout: 10_000 });

  // Click Edit on the card
  await page.locator(COMMENTS.editButton).first().click();
  await expect(page.locator(COMMENTS.editTextarea)).toBeVisible();

  const updated = `Updated ${Date.now()}`;
  await page.fill(COMMENTS.editTextarea, updated);
  await page.locator(COMMENTS.saveButton).click();

  // After save the edit textarea should be gone and the new content visible
  await expect(page.locator(COMMENTS.editTextarea)).not.toBeVisible();
  await expect(page.locator(COMMENTS.content)).toContainText(updated);
});

test('Ctrl+Enter submits the edit', async ({ page, createItem }) => {
  const title = uniqueTitle('CtrlEnterComment');
  await createItem(title);
  await navigateToItem(page, title);

  await page.fill(COMMENTS.draftTextarea, `Initial ${Date.now()}`);
  await page.click(COMMENTS.postButton);
  await expect(page.locator(COMMENTS.card)).toBeVisible({ timeout: 10_000 });

  await page.locator(COMMENTS.editButton).first().click();
  const edited = `Ctrl+Enter edit ${Date.now()}`;
  await page.fill(COMMENTS.editTextarea, edited);
  await page.keyboard.press('Control+Enter');

  await expect(page.locator(COMMENTS.editTextarea)).not.toBeVisible();
  await expect(page.locator(COMMENTS.content)).toContainText(edited);
});

test('delete a comment removes the card', async ({ page, createItem }) => {
  const title = uniqueTitle('DeleteComment');
  await createItem(title);
  await navigateToItem(page, title);

  // Post a comment
  await page.fill(COMMENTS.draftTextarea, `Delete me ${Date.now()}`);
  await page.click(COMMENTS.postButton);
  await expect(page.locator(COMMENTS.card)).toBeVisible({ timeout: 10_000 });

  // Delete it
  await page.locator(COMMENTS.deleteButton).first().click();

  // ConfirmDialog uses shadow DOM — Playwright auto-pierces open shadow roots
  await expect(page.locator(CONFIRM.dialog)).toBeVisible();
  await page.locator(CONFIRM.confirmButton).click();

  // Card should be removed; empty state should return
  await expect(page.locator(COMMENTS.card)).not.toBeVisible({ timeout: 10_000 });
  await expect(page.locator(COMMENTS.empty)).toBeVisible();
});

test('cancel edit restores the comment card', async ({ page, createItem }) => {
  const title = uniqueTitle('CancelEdit');
  await createItem(title);
  await navigateToItem(page, title);

  const original = `Cancel test ${Date.now()}`;
  await page.fill(COMMENTS.draftTextarea, original);
  await page.click(COMMENTS.postButton);
  await expect(page.locator(COMMENTS.card)).toBeVisible({ timeout: 10_000 });

  await page.locator(COMMENTS.editButton).first().click();
  await expect(page.locator(COMMENTS.editTextarea)).toBeVisible();

  await page.locator(COMMENTS.cancelEditButton).click();
  await expect(page.locator(COMMENTS.editTextarea)).not.toBeVisible();
  await expect(page.locator(COMMENTS.content)).toContainText(original);
});
