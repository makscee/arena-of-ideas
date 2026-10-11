# Arena in Claude Code

A Claude Code mod (#960): your run in a pane, played with keys, and the session's Claude as your coach. It calls the hosted API (`https://arena.makscee.ru`) directly; nothing runs locally.

```
/plugin install arena --marketplace makscee/arena-of-ideas
```

Answer `y` to add the marketplace, then pick a scope (user: every session, any folder). The marketplace clone is shallow (one commit). `/plugin update arena` takes a newer version.

```
Arena · Round 2/12 · ♥4 · 10g · 0W 1L · vs @bot-Esk   [ranked]
SHOP    1 Henchman T1 3g   2 Henchman T1 3g   3 Rose T1 3g   4 Wire T1 3g
LINE    1 💉 Nurse 1/5 1/3 · 2 🌹 Rose 2/6 1/3 · 3 🧨 Saboteur 2/4 1/3 · 4 — · 5 —
BENCH   6 — · 7 — · 8 —
LAST    round 1 vs @bot-Esk: loss (−1♥)
Claude suggests: buy 3 Rose (a second Rose toward Awoken)   y accept
1: buy 1-4  s: sell  r: reroll  l: lock  m: move  f: fuse  g: fight  b: battle  y: accept  t: RU/EN
```

- **Log in** once: `/arena login <email> [name]`, then `/arena code <code> [name]` (a new player needs a name). Or set `ARENA_TOKEN` (a session token). The login is kept across sessions, per server.
- `/arena` opens the pane on the run going (ranked first); `/arena practice` or `/arena ranked` opens or starts that run; `/arena ru` / `/arena en`; `/arena logout`.
- **Keys** (while the pane has focus; Esc goes back to the prompt, `/arena` brings the focus back). The numbers are the pane's, from 1:
  - `1`-`9` buy that shop offer (while a gift waits: take that gift, `x` skips it)
  - `s` then a board number sells; `l` then a shop number locks; `m` then two numbers moves; `f` then two fuses; `c` cancels
  - `r` reroll, `g` fight, `b` replay the last battle as text (`k`/`j` page, `b` back), `t` Russian/English
  - with no run going: `n` starts a ranked run, `p` a practice run
- **Claude** has arena-mcp's tools (#959, the same code, bundled): `home`, `run`, `preview`, `battle`, `trace`, `codex`, `start_run` and `decide` (practice runs only), `login`/`login_code`, plus `suggest {decision, why}`. Ask it "what should I buy?": its pick shows in the pane, and nothing moves until you press `y`. Run tools default to the run in the pane.
- **Assist only** (#955): your keys and `y` are your moves. Claude's tools send `X-Arena-Client: agent/claude-code`, so the server refuses Claude any ranked move; on a practice run Claude may play by itself.
- `ARENA_URL` points it at another server (e.g. a local one: `http://127.0.0.1:8791`).
- Don't also add arena-mcp under the name `arena` in the same Claude Code: both would be the `mcp__arena__*` tools.

Development, in a clone:

```sh
npm ci
npm run mod:build                         # hooks/arena.mjs, after changing src/, clients/core or clients/mcp/tools.ts
claude --plugin-dir clients/claude-code    # with ARENA_URL for a local server
claude plugin test clients/claude-code     # the pane in Claude Code's engine (tests/)
npx vitest run clients/claude-code         # keys against a real server, and hooks/arena.mjs is fresh (src/pane.spec.ts)
```

An installed mod has only this folder, so `hooks/arena.mjs` is a committed bundle of `src/lib.ts` (`scripts/mod-bundle.ts`); `src/pane.spec.ts` fails when it is stale.
