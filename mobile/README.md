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
| `npm run mvp:phone [-- --url …/arena/]` | a whole game in Chromium at 360×640, screenshots in `e2e/.shots/mvp`; without `--url` (its own local server) it also plays the champion's own Crown (`e2e/mvp-own-crown.ts`), a `--url` run skips that |
| `npm run mvp:desktop [-- --url …/arena/]` | a whole run at 1440×900 with mouse and keys only (hover, double-click, drag, 1–7, R, Space, ← →, F, S, Esc, Enter), awaken and fuse in the inspector, the desktop battle (facing lines, timeline drag, Log → Why, 1024×768), then the 1024px and 1023px edges; screenshots in `e2e/.shots/mvp-desktop` |
| `npm run mvp:check` | typecheck, tests, the bot, the phone run and the desktop run |
| `npm run mvp:redeploy [-- <branch>] [--fresh]` | redeploys the m1 test instance (default `mission-574-mvp`; usually `mvp-live`, the last checked build). It holds real players' data now: `--fresh` (an empty world) needs `ARENA_MVP_WIPE=1` and Maks's word |

Desktop (round 2, R2-9): at 1024px and wider a screen that calls
`screen(name)` (ui/dom.ts) gets its desktop layout from style.css; the shop
also swaps its pop-up sheets for an inspector on the right and takes keys
(`onKeys`). Below 1024px, and on screens without a name (the name screen),
it is the phone column. The battle (R2-16) is CSS only: the lines face each
other, a turn timeline scrubs, and a side panel holds Why and Log; the phone
keeps its stacked rows.

Test instance: https://m1.twin-pogona.ts.net/arena/ (tailnet only). It is a
launchd agent `ru.makscee.arena-mvp` in `~/arena-mvp` on m1, log in
`~/arena-mvp/data/server.log`, fronted by `tailscale serve --set-path /arena`.
State is a SQLite file, `~/arena-mvp/data/arena-mvp.db`: a plain redeploy
keeps it, and `ARENA_MVP_WIPE=1 npm run mvp:redeploy -- --fresh` (only on Maks's word: it holds real players now) moves it aside (to
`arena-mvp.db.bak-<time>`) so the server starts an empty world. It runs
with `MVP_DEV=1`, so the dev tools (`POST /api/v1/dev/end-day`) answer there;
without it they are 404.
