# Arena of Ideas

A game that makes itself — players create the content, the community curates it, the ladder selects what survives.

An async-PvP auto-battler where heroes and abilities are **data**, composed from atomic parts (triggers, interceptors, selectors, effects). Players create new parts through an LLM interface, a simulation gate checks balance, player votes judge fun, and fusion recombines everything at the table.

**This is v5** — a clean rebuild. The previous version (~3,800 commits of Rust/Bevy/SpacetimeDB) is preserved at tag [`v4-final`](../../tree/v4-final).

## Status

Building the headless battle kernel first: a pure, deterministic `battle(teamA, teamB, seed) → event log` with a causal trace — no client yet. See [SPEC.md](SPEC.md).

```sh
npm install
npm test
npm run typecheck
```

## Test scenes: every screen from one link

Every screen and state of the MVP opens from one link, with no playing to
reach it: `https://m1.twin-pogona.ts.net/arena-lab/#scene=<id>[&lang=en|ru][&theme=plum|brass|felt][&speed=1|2|4]`.
The index of them all is https://m1.twin-pogona.ts.net/r/arena-scenes.html (tailnet only).

- **Scenes** are rows in `server/src/mvp/scenes.ts`: a fixture built in code
  (a run played by the bot's rules to a round, a battle record, ideas, a day
  end) and the screen the client opens (`openScene` in `mobile/main.ts`).
  Only a server with `MVP_DEV=1 ARENA_SCENES=1 MVP_DB=:memory:` serves them;
  it refuses to start otherwise, so a scene never touches a real world. A
  scene tab keeps its player, language and palette in memory
  (`mobile/scene.ts`): the browser's saved login stays as it was.
- **The lab** (m1, launchd `ru.makscee.arena-lab`, `:8796`, `tailscale serve
  /arena-lab`) runs one branch: `npm run mvp:lab -- <branch>` (default `main`)
  checks it out in `~/arena-lab`, builds, restarts it and rewrites the index.
- **The check** `npm run mvp:scenes` opens every scene headless (phone EN and
  RU, desktop) and fails on an error or a blank screen; it is part of `mvp:check`.

**The rule:** a card that changes a screen adds or updates its scenes here,
and its How to try gives those scene links (point the lab at the card's
branch first: `npm run mvp:lab -- <branch>`).

## Deploy

One container ships the whole game: the arena server (`server/`, see
[server/README.md](server/README.md)) serves the vite-built web client
same-origin from `STATIC_DIR` — the client already talks relative `/v1/...`
URLs (vite proxies them in dev), so a single origin means no CORS anywhere.
The image runs TypeScript directly via tsx, mirroring the hub's void-mail
image; base is Debian slim (not alpine) because better-sqlite3 needs its
glibc prebuilds.

Build and run locally (no secrets needed in mock mode):

```sh
docker build -t ghcr.io/makscee/arena-of-ideas:latest .
MOCK_MODE=1 docker compose -f compose.dev.yml up --build
# game at http://localhost:8787 — MOCK_MODE mocks the mailer and mounts
# /_mock/last-code so login works without a mail service. Never set in prod.
```

Production follows the hub pattern (`compose.yml`, mirrors void-mail): the
image name is `ghcr.io/makscee/arena-of-ideas` (documented, not yet
published), Caddy reverse-proxies to the container on the shared `void`
network, no host port is mapped, and config is **env only** — see
[.env.example](.env.example). Secrets live in a SOPS-encrypted file decrypted
to `.env` at deploy time (`sops -d secrets.enc.yaml > .env`); only
`MAIL_BASE_URL` + `MAIL_TOKEN` are secret.

The one divergence from void-mail: state is a SQLite file, so the service
carries a named volume at `/data` (the image pre-chowns it for its non-root
user; named-volume init copies that ownership). Back it up by archiving the
volume:

```sh
docker run --rm -v arena-data:/data -v "$PWD":/out alpine \
  tar czf /out/arena-data.tgz -C /data .
```
