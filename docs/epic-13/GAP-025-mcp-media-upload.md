# GAP-025 — MCP cannot upload or rename files

## Problem

`apps/mcp/src/tools/media.rs` only exposes `list_media`, `create_media_folder`, and
`delete_media`. The HTTP API also has:
- `POST /media/*path` — file upload (multipart/form-data)
- `PATCH /media/rename` — rename a file or folder

Without upload capability, agents cannot attach images or files to items, cannot
create media for use in item bodies, and cannot manage the media library meaningfully.
(List and delete without upload is nearly useless.)

## Impact

- An agent asked to "save this image to my Zealot media library" cannot do it.
- An agent building a report item that should include an image cannot upload the image.
- Media rename is blocked: agents can create folders and upload files but cannot
  reorganise them.

## Proposed fix — `upload_media`

MCP tools cannot handle raw binary via multipart, but base64 is a standard workaround:

```rust
pub struct UploadMediaInput {
    /// Destination path including filename, e.g. "images/photo.jpg"
    pub path: String,
    /// File contents, base64-encoded
    pub content_base64: String,
}
```

The tool decodes the base64, then POSTs to `POST /media/{path}` as multipart.

**Alternative**: Since MCP HTTP clients can make arbitrary calls, expose upload as a
`request_upload_url` tool that returns the target URL + auth header, letting the
client do the upload directly.

## Proposed fix — `rename_media`

```rust
pub struct RenameMediaInput {
    pub old_location: String,  // current path
    pub new_name: String,      // new filename (not full path)
}
```

Calls `PATCH /media/rename` on the HTTP API.

## Files to change

- `apps/mcp/src/tools/media.rs` — add `upload_media` and `rename_media`
- `docs/mcp.md` — update Media tools table
