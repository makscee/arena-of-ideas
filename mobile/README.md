# Arena MVP: phone client (mission #574)

Vite + TypeScript, plain DOM, 360×640 first, the "B · Arena" palette.
Every slice codes against the contract in `src/mvp/contract.ts` (types and the
HTTP API list); the thin run lives in `src/mvp/run.ts`, the server in
`server/src/mvp/`.

Files: `api.ts` is the typed API client; `main.ts` holds the name, home,
shop and result screens (slice 8); `screens/battle.ts` is the battle viewer
(slice 9; a stub that goes straight to the result until then); `ui/dom.ts`
and `ui/card.ts` are the DOM helpers and the unit card both slices share.

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
State is in memory until slice 4: a redeploy starts an empty world.
