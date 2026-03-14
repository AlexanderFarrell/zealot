# zealot-mcp

A broad MCP server that exposes Zealot planning, item management, metadata management, comments, repeats, tracker data, analysis, and media operations to Claude and other MCP-compatible clients.

## Setup

```bash
go mod tidy
go build -o zealot-mcp .
```

## Runtime Modes

### Auto (default)

```bash
./zealot-mcp
```

Auto mode behavior:
- Uses `stdio` when stdin/stdout are piped (typical MCP client launch)
- Uses `http` when run interactively in a terminal

### HTTP

```bash
./zealot-mcp --transport=http --listen=:8080 --base-path=/mcp
```

- SSE endpoint: `/mcp/sse`
- Message endpoint: `/mcp/message`
- Health endpoint: `/healthz`

### Stdio

```bash
./zealot-mcp --transport=stdio
```

## Configuration

Required environment variables:

```bash
export ZEALOT_API_TOKEN=your_token_here
```

Optional environment variables:

- `ZEALOT_API_URL` (default: `https://zealot.alexanderfarrell.net`)
- `MCP_TRANSPORT` (default: `auto`, options: `auto`, `http`, `stdio`)
- `MCP_LISTEN_ADDR` (default: `:8080`)
- `MCP_BASE_PATH` (default: `/mcp`)
- `MCP_SSE_ENDPOINT` (default: `/sse`)
- `MCP_MESSAGE_ENDPOINT` (default: `/message`)

## Docker

Build:

```bash
docker build -t zealot-mcp .
```

Run:

```bash
docker run --rm -p 8080:8080 \
  -e ZEALOT_API_URL=https://zealot.alexanderfarrell.net \
  -e ZEALOT_API_TOKEN=your_token_here \
  zealot-mcp
```

## Claude Desktop Integration (Stdio)

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "zealot": {
      "command": "/path/to/zealot-mcp",
      "env": {
        "ZEALOT_API_URL": "https://zealot.alexanderfarrell.net",
        "ZEALOT_API_TOKEN": "your_token_here",
        "MCP_TRANSPORT": "stdio"
      }
    }
  }
}
```

> Auth note: the current client sends `Authorization: Bearer <token>`.
> If Zealot uses session-based auth (cookies) rather than bearer tokens, adjust `client.go`
> to use cookie-based auth or whatever header Zealot expects.

## Tool Coverage

The MCP now covers these categories:

- Capability discovery: `describe_backend_capabilities`
- Account access: details, API key status, settings updates
- Items: fetch, search, filter, create, update, delete, attribute mutation, type assignment
- Metadata: item types and attribute kinds, including create/update/delete flows
- Planner and repeats: day/week/month/year planner queries plus repeat status updates
- Comments, tracker, and analysis: comment CRUD, tracker entry CRUD, score history
- Media: directory listing, folder creation, upload, rename, and delete

Use `describe_backend_capabilities` from your MCP client if you want the live route-to-tool map.

## Filter Examples

```json
// This week's items
[{"key": "Week", "op": "eq", "value": "202610"}]

// High priority To Do items
[{"key": "Priority", "op": "gte", "value": 7}, {"key": "Status", "op": "eq", "value": "To Do"}]

// Books not yet complete
[{"key": "Status", "op": "ne", "value": "Complete"}]
```

## Auth Caveat

Check `zealotd/apps/account/` to confirm how Zealot expects authentication.
If it's cookie-session based (likely, given it's a SPA with `RequireLoginMiddleware`), you'll need to either:
1. Add an API key mechanism to Zealot (recommended; a static token for MCP is fine)
2. Capture and pass a session cookie

Option 1 is cleaner and keeps the MCP server stateless.
