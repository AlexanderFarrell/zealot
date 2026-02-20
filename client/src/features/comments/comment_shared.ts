import { DateTime } from "luxon";
import API from "../../api/api";
import type { CommentEntry } from "../../api/comment";
import createZealotEditorView from "../zealotscript/zealotscript_editor";

export function sortComments(entries: CommentEntry[]) {
	return [...entries].sort((a, b) => {
		const aTime = DateTime.fromISO(a.timestamp).toMillis();
		const bTime = DateTime.fromISO(b.timestamp).toMillis();
		return bTime - aTime;
	});
}

export function emitCommentsLoaded(target: HTMLElement, entries: CommentEntry[]) {
	target.dispatchEvent(
		new CustomEvent("comments-loaded", {
			detail: { entries }
		})
	);
}

export function createCommentEditor(entry: CommentEntry) {
	const comment = document.createElement("div");
	comment.classList.add("comment-editor");
	createZealotEditorView(comment, {
		content: entry.content || "",
		debounceMs: 500,
		handleTab: true,
		onUpdate: async (nextContent) => {
			try {
				entry.content = nextContent;
				await API.comment.update_entry(entry.comment_id, nextContent);
			} catch (e) {
				console.error(e);
			}
		}
	});
	return comment;
}

export function createDeleteButton(commentID: number, onDeleteSuccess: () => Promise<void>) {
	const del = document.createElement("button");
	del.innerText = "Delete";
	del.addEventListener("click", async () => {
		try {
			await API.comment.delete_entry(commentID);
			await onDeleteSuccess();
		} catch (e) {
			console.error(e);
		}
	});
	return del;
}
