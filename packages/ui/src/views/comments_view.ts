import { Popups, getNavigator } from '@websoil/engine';
import { AuthAPI } from '@zealot/api/src/auth';
import { CommentAPI } from '@zealot/api/src/comment';
import type { Comment } from '@zealot/domain/src/comment';
import type { Item } from '@zealot/domain/src/item';
import { DateTime } from 'luxon';
import { ConfirmDialog } from '../common/confirm_dialog';
import { LoadingSpinner } from '../common/loading_spinner';
import { ItemSearchInline } from './item_search_inline';

const authApi = new AuthAPI('/api');
const commentApi = new CommentAPI('/api');
let currentUsernamePromise: Promise<string> | null = null;

export type CommentsViewScope =
    | { kind: 'day'; date: DateTime }
    | { kind: 'item'; itemId: number };

export interface CommentsViewConfig {
    scope: CommentsViewScope;
}

function currentTimeValue(): string {
    return DateTime.local().toFormat('HH:mm');
}

function extractMessage(error: unknown, fallback: string): string {
    return error instanceof Error && error.message ? error.message : fallback;
}

function sortComments(entries: Comment[]): Comment[] {
    return [...entries].sort((a, b) => a.Timestamp.toMillis() - b.Timestamp.toMillis());
}

async function getCurrentUsername(): Promise<string> {
    if (!currentUsernamePromise) {
        currentUsernamePromise = (async () => {
            try {
                await authApi.IsLoggedIn();
                return authApi.Account?.Username ?? 'You';
            } catch {
                return 'You';
            }
        })();
    }

    return currentUsernamePromise;
}

export class CommentsView extends HTMLElement {
    private _authorLabel = 'You';
    private _comments: Comment[] = [];
    private _config: CommentsViewConfig | null = null;
    private _deletingCommentId: number | null = null;
    private _draftContent = '';
    private _draftError: string | null = null;
    private _draftItem: Item | null = null;
    private _draftTime = currentTimeValue();
    private _editingCommentId: number | null = null;
    private _editingContent = '';
    private _loadError: string | null = null;
    private _loading = false;
    private _requestId = 0;
    private _savingCommentId: number | null = null;
    private _submitting = false;

    connectedCallback(): void {
        if (!this._config) {
            return;
        }

        this._render();
        void this._ensureAuthor();
        if (!this._loading && this._comments.length === 0 && this._loadError == null) {
            void this._loadComments();
        }
    }

    init(config: CommentsViewConfig): this {
        this._config = config;
        this._comments = [];
        this._deletingCommentId = null;
        this._draftContent = '';
        this._draftError = null;
        this._draftItem = null;
        this._draftTime = currentTimeValue();
        this._editingCommentId = null;
        this._editingContent = '';
        this._loadError = null;
        this._loading = false;
        this._savingCommentId = null;
        this._submitting = false;
        this._requestId += 1;
        this._render();

        if (this.isConnected) {
            void this._ensureAuthor();
            void this._loadComments();
        }

        return this;
    }

    private async _ensureAuthor(): Promise<void> {
        const username = await getCurrentUsername();
        if (this._authorLabel === username) {
            return;
        }

        this._authorLabel = username;
        this._render();
    }

    private async _loadComments(): Promise<void> {
        if (!this._config) {
            return;
        }

        const requestId = ++this._requestId;
        this._loading = true;
        this._loadError = null;
        this._render();

        try {
            const entries = this._config.scope.kind === 'day'
                ? await commentApi.GetForDay(this._config.scope.date)
                : await commentApi.GetForItem(this._config.scope.itemId);

            if (requestId !== this._requestId) {
                return;
            }

            this._comments = sortComments(entries);
        } catch (error) {
            if (requestId !== this._requestId) {
                return;
            }
            this._loadError = extractMessage(error, 'Failed to load comments.');
        } finally {
            if (requestId === this._requestId) {
                this._loading = false;
                this._render();
            }
        }
    }

    private _render(): void {
        if (!this._config) {
            this.innerHTML = '';
            return;
        }

        this.className = 'comments-view';
        this.innerHTML = '';

        const shell = document.createElement('div');
        shell.className = 'comments-view-shell';

        const list = document.createElement('div');
        list.className = 'comments-view-list';

        if (this._loading && this._comments.length === 0) {
            const loading = document.createElement('div');
            loading.className = 'comments-view-loading';
            loading.appendChild(new LoadingSpinner());
            list.appendChild(loading);
        } else if (this._loadError) {
            const error = document.createElement('p');
            error.className = 'tool-error comments-view-error';
            error.textContent = this._loadError;
            list.appendChild(error);
        } else if (this._comments.length === 0) {
            const empty = document.createElement('p');
            empty.className = 'tool-muted comments-view-empty';
            empty.textContent = 'No comments yet.';
            list.appendChild(empty);
        } else {
            this._comments.forEach((comment) => {
                list.appendChild(this._buildCommentCard(comment));
            });
        }

        shell.append(list, this._buildComposer());
        this.appendChild(shell);
    }

    private _buildCommentCard(comment: Comment): HTMLElement {
        const card = document.createElement('article');
        card.className = 'comments-view-card';

        const header = document.createElement('div');
        header.className = 'comments-view-card-header';

        const meta = document.createElement('div');
        meta.className = 'comments-view-meta';

        const author = document.createElement('span');
        author.className = 'comments-view-author';
        author.textContent = this._authorLabel;
        meta.appendChild(author);

        const timestamp = document.createElement('span');
        timestamp.className = 'comments-view-timestamp';
        timestamp.textContent = this._formatTimestamp(comment.Timestamp);
        meta.appendChild(timestamp);

        if (this._config?.scope.kind === 'day') {
            const item = document.createElement('button');
            item.type = 'button';
            item.className = 'comments-view-item';
            item.textContent = comment.Item.DisplayTitle;
            item.addEventListener('click', () => {
                getNavigator().openItemById(comment.Item.ItemID);
            });
            meta.appendChild(item);
        }

        const actions = document.createElement('div');
        actions.className = 'comments-view-actions';

        if (this._editingCommentId === comment.CommentID) {
            const cancel = document.createElement('button');
            cancel.type = 'button';
            cancel.textContent = 'Cancel';
            cancel.disabled = this._savingCommentId === comment.CommentID;
            cancel.addEventListener('click', () => {
                this._editingCommentId = null;
                this._editingContent = '';
                this._savingCommentId = null;
                this._render();
            });

            const save = document.createElement('button');
            save.type = 'button';
            save.textContent = this._savingCommentId === comment.CommentID ? 'Saving...' : 'Save';
            save.disabled = this._savingCommentId === comment.CommentID;
            save.addEventListener('click', () => {
                void this._saveComment(comment);
            });

            actions.append(cancel, save);
        } else {
            const edit = document.createElement('button');
            edit.type = 'button';
            edit.textContent = 'Edit';
            edit.disabled = this._deletingCommentId === comment.CommentID;
            edit.addEventListener('click', () => {
                this._editingCommentId = comment.CommentID;
                this._editingContent = comment.Content;
                this._render();
                this._focusEdit(comment.CommentID);
            });

            const del = document.createElement('button');
            del.type = 'button';
            del.textContent = this._deletingCommentId === comment.CommentID ? 'Deleting...' : 'Delete';
            del.disabled = this._deletingCommentId === comment.CommentID;
            del.addEventListener('click', () => {
                void this._deleteComment(comment);
            });

            actions.append(edit, del);
        }

        header.append(meta, actions);

        const body = document.createElement('div');
        body.className = 'comments-view-body';

        if (this._editingCommentId === comment.CommentID) {
            const textarea = document.createElement('textarea');
            textarea.className = 'comments-view-editor';
            textarea.value = this._editingContent;
            textarea.dataset.commentEditId = String(comment.CommentID);
            textarea.rows = Math.max(3, comment.Content.split('\n').length);
            textarea.addEventListener('input', () => {
                this._editingContent = textarea.value;
            });
            textarea.addEventListener('keydown', (event) => {
                if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
                    event.preventDefault();
                    void this._saveComment(comment);
                }
            });
            body.appendChild(textarea);
        } else {
            const content = document.createElement('p');
            content.className = 'comments-view-content';
            content.textContent = comment.Content;
            body.appendChild(content);
        }

        card.append(header, body);
        return card;
    }

    private _buildComposer(): HTMLElement {
        const composer = document.createElement('form');
        composer.className = 'comments-view-composer';
        composer.addEventListener('submit', (event) => {
            event.preventDefault();
            void this._submitComment();
        });

        const heading = document.createElement('h3');
        heading.textContent = 'Add Comment';
        composer.appendChild(heading);

        if (this._draftError) {
            const error = document.createElement('p');
            error.className = 'tool-error comments-view-error';
            error.textContent = this._draftError;
            composer.appendChild(error);
        }

        if (this._config?.scope.kind === 'day') {
            const itemField = document.createElement('label');
            itemField.className = 'tool-field';

            const itemLabel = document.createElement('span');
            itemLabel.className = 'tool-label';
            itemLabel.textContent = 'Item';
            itemField.appendChild(itemLabel);

            const picker = new ItemSearchInline();
            picker.placeholder = 'Search item…';
            picker.value = this._draftItem;
            picker.OnSelect = (item) => {
                this._draftItem = item;
                this._draftError = null;
            };
            itemField.appendChild(picker);

            composer.appendChild(itemField);

            const timeField = document.createElement('label');
            timeField.className = 'tool-field';

            const timeLabel = document.createElement('span');
            timeLabel.className = 'tool-label';
            timeLabel.textContent = 'Time';
            timeField.appendChild(timeLabel);

            const time = document.createElement('input');
            time.type = 'time';
            time.value = this._draftTime;
            time.className = 'comments-view-time-input';
            time.addEventListener('input', () => {
                this._draftTime = time.value;
            });
            timeField.appendChild(time);

            composer.appendChild(timeField);
        }

        const contentField = document.createElement('label');
        contentField.className = 'tool-field';

        const contentLabel = document.createElement('span');
        contentLabel.className = 'tool-label';
        contentLabel.textContent = 'Content';
        contentField.appendChild(contentLabel);

        const textarea = document.createElement('textarea');
        textarea.className = 'comments-view-editor comments-view-composer-editor';
        textarea.placeholder = 'Write a comment…';
        textarea.value = this._draftContent;
        textarea.dataset.commentsDraft = 'true';
        textarea.rows = 4;
        textarea.addEventListener('input', () => {
            this._draftContent = textarea.value;
            this._draftError = null;
        });
        textarea.addEventListener('keydown', (event) => {
            if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
                event.preventDefault();
                void this._submitComment();
            }
        });
        contentField.appendChild(textarea);

        composer.appendChild(contentField);

        const actions = document.createElement('div');
        actions.className = 'comments-view-composer-actions';

        const submit = document.createElement('button');
        submit.type = 'submit';
        submit.textContent = this._submitting ? 'Posting...' : 'Post';
        submit.disabled = this._submitting;
        actions.appendChild(submit);

        composer.appendChild(actions);
        return composer;
    }

    private async _submitComment(): Promise<void> {
        if (!this._config || this._submitting) {
            return;
        }

        const content = this._draftContent.trim();
        if (!content) {
            this._draftError = 'Comment content is required.';
            this._render();
            this._focusDraft();
            return;
        }

        const itemId = this._config.scope.kind === 'item'
            ? this._config.scope.itemId
            : this._draftItem?.ItemID;

        if (itemId == null) {
            this._draftError = 'Select an item before posting a planner comment.';
            this._render();
            return;
        }

        const timestamp = this._resolveDraftTimestamp();
        if (!timestamp) {
            this._draftError = 'Enter a valid time before posting.';
            this._render();
            return;
        }

        this._submitting = true;
        this._draftError = null;
        this._render();

        try {
            const created = await commentApi.AddComment({
                content,
                item_id: itemId,
                timestamp,
            });

            this._comments = sortComments([...this._comments, created]);
            this._draftContent = '';
            this._draftError = null;
            if (this._config.scope.kind === 'day') {
                this._draftTime = currentTimeValue();
                this._draftItem = null;
            }
        } catch (error) {
            const message = extractMessage(error, 'Failed to post comment.');
            this._draftError = message;
            Popups.add_error(message);
        } finally {
            this._submitting = false;
            this._render();
            this._focusDraft();
        }
    }

    private async _saveComment(comment: Comment): Promise<void> {
        const content = this._editingContent.trim();
        if (!content || this._savingCommentId === comment.CommentID) {
            return;
        }

        this._savingCommentId = comment.CommentID;
        this._render();

        try {
            const updated = await commentApi.UpdateEntry({
                comment_id: comment.CommentID,
                content,
            });

            this._comments = this._comments.map((entry) =>
                entry.CommentID === updated.CommentID ? updated : entry,
            );
            this._editingCommentId = null;
            this._editingContent = '';
        } catch (error) {
            Popups.add_error(extractMessage(error, 'Failed to save comment.'));
        } finally {
            this._savingCommentId = null;
            this._render();
        }
    }

    private async _deleteComment(comment: Comment): Promise<void> {
        if (this._deletingCommentId === comment.CommentID) {
            return;
        }

        const confirmed = await ConfirmDialog.show('Are you sure you want to delete this comment?');
        if (!confirmed) {
            return;
        }

        this._deletingCommentId = comment.CommentID;
        this._render();

        try {
            await commentApi.DeleteEntry(comment.CommentID);
            this._comments = this._comments.filter((entry) => entry.CommentID !== comment.CommentID);

            if (this._editingCommentId === comment.CommentID) {
                this._editingCommentId = null;
                this._editingContent = '';
            }
        } catch (error) {
            Popups.add_error(extractMessage(error, 'Failed to delete comment.'));
        } finally {
            this._deletingCommentId = null;
            this._render();
        }
    }

    private _resolveDraftTimestamp(): DateTime | null {
        if (!this._config) {
            return null;
        }

        if (this._config.scope.kind === 'item') {
            return DateTime.local().startOf('minute');
        }

        const match = /^(?<hour>\d{2}):(?<minute>\d{2})$/.exec(this._draftTime);
        if (!match?.groups) {
            return null;
        }

        const hour = Number(match.groups.hour);
        const minute = Number(match.groups.minute);
        if (!Number.isInteger(hour) || !Number.isInteger(minute)) {
            return null;
        }

        const timestamp = this._config.scope.date.set({
            hour,
            minute,
            second: 0,
            millisecond: 0,
        });

        return timestamp.isValid ? timestamp : null;
    }

    private _formatTimestamp(timestamp: DateTime): string {
        if (!timestamp.isValid) {
            return 'Invalid time';
        }

        if (this._config?.scope.kind === 'day') {
            return timestamp.toFormat('HH:mm');
        }

        return timestamp.toFormat('dd LLL yyyy HH:mm');
    }

    private _focusDraft(): void {
        window.requestAnimationFrame(() => {
            const textarea = this.querySelector<HTMLTextAreaElement>('[data-comments-draft="true"]');
            textarea?.focus();
        });
    }

    private _focusEdit(commentId: number): void {
        window.requestAnimationFrame(() => {
            const textarea = this.querySelector<HTMLTextAreaElement>(`[data-comment-edit-id="${commentId}"]`);
            textarea?.focus();
            textarea?.setSelectionRange(textarea.value.length, textarea.value.length);
        });
    }
}

if (!customElements.get('comments-view')) {
    customElements.define('comments-view', CommentsView);
}
