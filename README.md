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
