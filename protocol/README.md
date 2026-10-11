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

## Agents: assist only

An agent client (an MCP server, the Claude Code mod, any bot acting for a player) sends
`X-Arena-Client: agent/<name>` (`client` in `clients/core`'s `Connection`). With it the server
allows reads, `preview` of any move, logging in, and practice runs; a move that counts needs a
human's tap, so every other write answers `403`: a ranked start, decide or abandon, ideas, votes,
linking a login, the dev tools. Which writes are open is `agent` in `src/mvp/routes.ts`; the
OpenAPI file lists each route's `403`.

A practice run is `POST /runs {"mode": "practice"}`. It plays the same shops, ghosts and Crown,
and counts for nothing: no rating, never a Slay (so never the throne), no ghost saved, no stats,
no fusion credit (a pair nobody has fused shows a provisional name). A player may have one ranked
and one practice run going; `GET /home` shows both (`activeRunId`, `practiceRunId`).

Until per-player API tokens (mission #955, slice 5) this is an honest-client rule: our clients
send the header, a hand-written one could leave it out.
