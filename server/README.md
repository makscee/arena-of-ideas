# Arena server

Standalone server for arena-of-ideas: email-OTP accounts and the shared
ladder API. Hono + Drizzle + better-sqlite3, run with tsx, tested with
vitest — same toolchain as the kernel, which the server imports directly
(same repo, no version skew).

The OTP/session services are copied from void-auth (pinned decision: copy,
don't call) and keep its security properties: codes and tokens are stored
sha256-only, compared timing-safe; codes live 10 minutes and die after 5 wrong
attempts; sessions are 32-byte bearer tokens with a 30-day lifetime.

## Run

```sh
npm install                                 # repo root; server is a workspace
MAIL_BASE_URL=https://mail.example.com \
MAIL_TOKEN=secret \
npm start --workspace server
```

## Env vars

| Var | Required | Default | Meaning |
| --- | --- | --- | --- |
| `MAIL_BASE_URL` | yes | — | void-mail base URL (codes go out via `POST /v1/mail/send`) |
| `MAIL_TOKEN` | yes | — | void-mail bearer token |
| `DB_PATH` | no | `./data/arena.db` | SQLite file (created on boot) |
| `PORT` | no | `8787` | listen port |

## Endpoints

- `POST /v1/auth/login/email/start` `{email}` → `{sent:true}` — always 200 for
  a valid email (no existence enumeration); rate-limited 5/IP and 5/email per
  10 minutes (429).
- `POST /v1/auth/login/email/verify` `{email, code}` → `{token, sessionId,
  expiresAt}` — mints a session, auto-creates the user on first login
  (`displayName` null until the first-login name pick).
- `POST /v1/auth/logout` (bearer) → `{ok:true}` — revokes the session.
- `GET /v1/auth/me` (bearer) → session + user.
- `GET /healthz` → `{ok:true}`.
- `GET /v1/ladder/champion` → `{champion, holder}` — **public**: the title
  screen shows the leaderboard to logged-out players, so reads need no login.
  `holder` is the owning user's display name (null for the bootstrap seat).
- `GET /v1/ladder/pool/:round` → `{round, pool}` — **public** full pool; with
  `?exclude=me` (bearer required, 401 without) the pool as the caller's runs
  see it: every ghost that user owns is filtered out. Play reads use the
  filtered form; display reads use the public one.
- `POST /v1/runs` (bearer) `{run: serializeRun(state)}` → 200
  `{accepted:true, runId, endedBy, finalRound, crowned}` or 422
  `{accepted:false, reason}` — submit a finished run for re-derivation.

## The shared ladder

One ladder per server instance, stored in SQLite behind the kernel's
`LadderStore` interface (`ladder-store.ts`) with the same semantics the
kernel's backings pin (append-only pools, the seq precondition, snapshot
isolation). Opened from the kernel's bootstrap at boot — `BOOTSTRAP_CHAMPION`
is seated, because a vacant champion spot is a free crown — and never reseeded
once played on.

**Users own ghosts.** A user is identified by the session's user id, and a
user's ghosts span all their runs: their own draws (`?exclude=me`, and the
submission replay) never contain them — the kernel's own-ghost rule lifted
from run level to user level. runIds are globally one-shot (`run_submissions`
PK), so a kernel-level runId collision across users cannot happen; clients
should mint unique runIds.

**Runs are re-derived, never trusted** (`runs.ts`). A submission is the
kernel's `serializeRun` output: seed + decision log + claimed final state, all
by value. The server pins pool/statuses to the arena's shipped content
(`content.ts` — DEFAULT_RUN_POOL + the committed approved registry), recovers
the decision sequence from the run log, and replays it through the kernel's
pure transitions. Ladder fights replay against the historical view the log
itself pins: pools are append-only and the server is the only writer, so what
a client saw is always a prefix of the user-filtered pool — each `Snapshotted`
seq fixes that prefix's length, and each `ChampionChallenged` names a champion
the append-only history can still produce. The re-derived state must match the
claim exactly (final stats, lives, every event); only re-derived ghosts/crowns
are written, re-sequenced onto the end of the current pools. Inflated stats,
fabricated wins, illegal decisions, wrong seeds, foreign content,
resubmissions: all 422 with the reason. A submission that replays against an
old-but-real prefix is accepted (it is a run that legally happened); staleness
bounds are a later concern.

**The crown race.** Two runs can legally beat the same champion. The first
submission takes the spot; a later submission whose beaten champion has since
been dethroned still lands its ghosts but the crown lapses (`crowned: false`).

## Test

`npm test` at the repo root runs the whole suite, server contract tests
included (in-process Hono app, in-memory SQLite, mock mailer — no network).
`npm test --workspace server` runs just these.
