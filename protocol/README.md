# Arena protocol

Arena is a game any client can play: the phone app, `npm run mvp:bot`, an agent through MCP, a
Claude Code pane. They all speak this protocol.

| File | What | Served at |
|---|---|---|
| [`openapi.json`](openapi.json) | Every route, its body and its answer (OpenAPI 3.1) | `GET /api/v1/openapi.json` |
| [`schema.json`](schema.json) | Every request and response type (JSON Schema) | `GET /api/v1/schema.json` |
| [`battle-events.md`](battle-events.md) | `battle-events/1`: the battle log a client replays | |

Both JSON files are generated from the TypeScript: the types in `src/mvp/contract.ts` and the
route table in `src/mvp/routes.ts`. After changing either, run `npm run protocol`; a test fails
while they're stale.

A client that reads battles and cards without the DOM or Node starts from `clients/core/`.

## The promise

- **`/api/v1` only gains things:** new routes, new fields, new values. Renaming or removing a
  route, a field or a value, or changing what one means, is `/api/v2`. `GET /api/v1/health`
  answers `api: 1`.
- **`battle-events/1` only gains things** the same way; a client skips an event type it doesn't
  know and ignores fields it doesn't know.
- Fields a client doesn't know are ignored, never an error, on both sides.

## Identity

Send `X-Arena-Token: <token>` from a `PlayerSession` (invite link, join link, email or Telegram
login). On an open dev server, `POST /players {name}` returns an id to send as `X-Arena-Player`.
Errors are `{ "error": string }` with a 4xx or 5xx status.
