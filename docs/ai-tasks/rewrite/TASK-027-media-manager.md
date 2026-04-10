# TASK-027: Media Manager Screen

## Context
The media manager is a file browser and uploader accessible at `/media/*`. It allows navigating folders, uploading files, renaming, and deleting.

Run `git diff master -- client/src/features/media/media_screen.ts` to see the original.

## Goal
Build the media manager screen.

## Requirements

### Layout
- Breadcrumb navigation bar showing current path (each segment clickable to navigate up)
- File/folder grid or list view
- Toolbar: "New Folder" button, "Upload" button

### File Listing
- Calls `GET /api/media/:location` with the current path
- Shows folders and files with appropriate icons
- File type icons for common types (image, document, archive, etc.)
- Folders are navigable (updates URL and reloads listing)

### Upload
- Click "Upload" opens a file picker (or drag-and-drop onto the grid)
- Calls `POST /api/media/:location` with multipart form data
- Shows upload progress; refreshes listing on completion

### New Folder
- Click "New Folder" → inline text input for folder name
- Calls `POST /api/media/mkdir` with `{ location: currentPath + "/" + name }`

### Rename
- Right-click or action button on a file/folder → rename
- Calls `PATCH /api/media/rename` with `{ old_path, new_path }`

### Delete
- Action button on a file/folder → confirmation → `DELETE /api/media/:path`

### Download
- Clicking a file downloads it as a blob (`GET /api/media/:location`)

## Dependencies
- TASK-001 (Router — path is part of the URL)

## Files Likely Involved
- `packages/ui/src/` (new `media_screen.ts`)
- `packages/api/src/media.ts`
