# Arena MVP: phone client (mission #574)

Vite + TypeScript, plain DOM, 360×640 first, the "B · Arena" palette.
Every slice codes against the contract in `src/mvp/contract.ts` (types and the
HTTP API list); the thin run lives in `src/mvp/run.ts`, the server in
`server/src/mvp/`.

Files and owners:

| File | Owner | What |
| --- | --- | --- |
| `main.ts` | slice 8 | name, home, shop and result screens; Home also shows the last playoff and, on a dev server, "End day now" |
| `screens/battle.ts` | slice 9 | the battle viewer and "why I lost" (stubs: straight to the result, no card) |
| `screens/stats.ts` | slice 11 | the stats page Home links to (a stub) |
| `ui/card.ts` | slice 8 | the unit card and the unit sheet with both forms; 9 and 11 only pass options |
| `ui/unit-stats.ts` | slice 11 | win and pick rate on every card and sheet (null until then) |
| `ui/dom.ts`, `api.ts`, `content.ts` | shared | DOM helpers and `overlay()`, the typed API client, the content pack loaded once |

| Command | What it does |
| --- | --- |
| `npm run mvp:server` | API + built client on 127.0.0.1:8791 at `/arena/` |
| `npm run mvp:dev` | Vite dev server, API proxied to :8791 |
| `npm run mvp:build` | builds `mobile/dist` |
| `npm run mvp:bot -- --runs 50 [--url …/arena]` | 50 full runs through the HTTP API, fails on any error |
| `npm run mvp:phone [-- --url …/arena/]` | a whole game in Chromium at 360×640, screenshots in `e2e/.shots/mvp` |
| `npm run mvp:check` | typecheck, tests, the bot and the phone run |
| `npm run mvp:redeploy [-- <branch>]` | redeploys the m1 test instance (default `mission-574-mvp`) |

Test instance: https://m1.twin-pogona.ts.net/arena/ (tailnet only). It is a
launchd agent `ru.makscee.arena-mvp` in `~/arena-mvp` on m1, log in
`~/arena-mvp/data/server.log`, fronted by `tailscale serve --set-path /arena`.
State is in memory until slice 4: a redeploy starts an empty world. It runs
with `MVP_DEV=1`, so the dev tools (`POST /api/v1/dev/end-day`) answer there;
without it they are 404.
