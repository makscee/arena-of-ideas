# arena-mcp

Arena as MCP tools (#959): any agent (Claude, Codex, Pi) can coach, or play practice runs. A stdio server on `clients/core`, bundled into one file that the game server serves at `/arena-mcp.mjs`.

```
curl -fsSL https://arena.makscee.ru/arena-mcp.mjs -o ~/.arena-mcp.mjs
claude mcp add arena -- node ~/.arena-mcp.mjs
```

| tool | does |
| --- | --- |
| `login {email}` / `login_code {code, name?}` | today's email login; the session is saved in `~/.config/arena-mcp/session.json` (0600) |
| `home` | rating, today's champion, your runs, ideas held |
| `start_run` | a practice run (or the one going) |
| `run {runId?}` | round, ♥, gold, the numbered SHOP, LINE, BENCH, gift, next opponent |
| `decide {runId?, decision}` | one move in a practice run; a ranked run answers "needs your tap" and moves nothing |
| `preview {runId?, decision}` | what a move would do, on any run, no writes |
| `battle {battleId, view?, lang?}` | a battle as text (`summary` or `steps`, rows end `#id`) |
| `trace {battleId, eventId, lang?}` | why an event happened |
| `codex {query?, lang?}` | unit cards, terms, `glossary` |

`lang: "ru"` reads Russian. Every request sends `X-Arena-Client: agent/mcp`, so the server itself refuses ranked moves (#958).

Env: `ARENA_URL` (default `https://arena.makscee.ru`, the server's root), `ARENA_TOKEN` (a session token: skips login), `ARENA_SESSION_FILE`.

Build: `npm run mcp:build -- out.mjs`; `npm run mvp:build` puts it in `mobile/dist/arena-mcp.mjs`. `mcp.test.ts` drives the bundle over stdio against a real server through a whole practice run.
