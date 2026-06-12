# Arena server

Standalone server for arena-of-ideas: email-OTP accounts and the shared
ladder API. Hono + Drizzle + better-sqlite3, run with tsx, tested with
vitest — same toolchain as the kernel, which the server imports directly
(same repo, no version skew).

The OTP/session services are copied from void-auth (pinned decision: copy,
don't call) and keep its security properties: codes and tokens are stored
sha256-only, compared timing-safe; codes live 10 minutes and die after 5 wrong
attempts; sessions are 32-byte bearer tokens with a 30-day lifetime. A
5-minute interval sweep (`cleanup.ts`, the void-auth pattern) prunes expired
sessions and hour-stale email codes.

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
| `PORT` | no | `8787` | listen port — a non-integer or out-of-range value fails boot |

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
  filtered form; display reads use the public one. `:round` is bounded
  (1..10000) — anything else is 400.
- `POST /v1/runs/open` (bearer) `{runId}` → 200 `{opened:true, runId}` or 422
  `{opened:false, reason}` — open a run **before playing it**. One-shot per
  runId (1–128 chars, `bootstrap` reserved; a submitted runId never reopens),
  and opens **expire 14 days after opening** (`RUN_OPEN_TTL_DAYS`).
- `GET /v1/runs/:runId/pool/:round` (bearer) → 200 `{served:true, round,
  pool, champion}` or 422 `{served:false, reason}` — **the play read**: the
  round's pool as this run's owner sees it (own ghosts excluded) plus the
  champion seated right now. The server **records every view it serves**
  (length + champion, per runId and round), and submission replay accepts
  only recorded views — so every fight of a run to be submitted must read
  through this endpoint. Re-reads are free and never brick a submission.
  Refused for runs not opened by the caller and for expired opens.
- `POST /v1/runs` (bearer) `{run: serializeRun(state)}` → 200
  `{accepted:true, runId, endedBy, finalRound, crowned}` or 422
  `{accepted:false, reason}` — submit a finished run for re-derivation. The
  runId must have been opened by the same user, within the open TTL.
  Cost-bounded before replay: max 256 KiB serialized, max 5000 log events.

**Client flow** (the contract the slice-3 web backing implements):

1. `POST /v1/runs/open` with a fresh unique runId, right when the run starts.
2. Per ladder fight at round R: `GET /v1/runs/:runId/pool/:round` and build
   the kernel's per-fight `LadderStore` view from **that one response** —
   `poolAt(R)` returns `pool`, `champion()` returns `champion`. Do not mix a
   pool from one read with a champion from another (the leaderboard reads
   below are for display only): a challenge replays only against the champion
   co-served with the claimed pool view.
3. Submit the finished run within the 14-day open TTL.

Skipping the open, or fighting against any view the server did not serve for
that runId, makes the submission unverifiable and it is rejected.

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
resubmissions: all 422 with the reason.

**Serve-time pinning: replay accepts only views the server handed out.** A
`Snapshotted` seq is client-claimed, and forgeable: claiming a shorter prefix
than the player observed cherry-picks the deterministic opponent draw, and
claiming an empty pool turns the kernel's outran-every-ghost rule into a free
champion challenge. Window checks are not enough — bounding each claim to
`[visible at open, visible now]` leaves the open itself bankable: an open
taken at ladder genesis and cashed in much later legally claims genesis
prefixes, dodges every ghost that landed since, and converts the genesis-empty
round 4 into a free challenge against *today's* champion (the banked-open
forgery that killed the previous build). So nothing is trusted that the server
did not itself serve: every play read (`GET /v1/runs/:runId/pool/:round`) is
**recorded** — runId, round, served prefix length, and the champion seated at
that moment (`run_pool_serves`) — and replay holds the run to that record. A
claimed seq must *equal* a served length for that (runId, round), and a
champion challenge must name the champion **co-served** with that very view: a
run cannot fight the past's pool against the present's champion. Honest play
is untouched — each fight's view is one serve, pools growing mid-run just mean
different serves at different lengths, re-reads add rows (any of them
replays), and a slow run resumed days later replays the views it actually
fetched. What banking still buys is exactly what honest slow play would get:
a challenge against the long-dethroned champion co-served back then, whose
crown lapses under the crown-race rule below.

Two supplements bound the residue. **Opens expire** (14 days,
`RUN_OPEN_TTL_DAYS` in `runs.ts`) — generously above an honest run's lifetime,
so it only kills parked opens; expired opens neither serve nor submit, and the
cleanup sweep prunes them with their serves (serves of submitted runs go too —
a runId never replays twice). The open row also records the ladder's ghost
watermark as provenance, though replay relies on the strictly stronger serve
record. A third remedy was considered and **rejected as primary**: flooring
champion-challenge rounds to the ladder's current first-empty round. It blocks
the genesis-challenge trick but still allows within-window cherry-picking at
earlier rounds, and it would falsely reject an honest challenge made just
before another ghost landed at that round — serve-time pinning has neither
problem.

**The crown race.** Two runs can legally beat the same champion. The first
submission takes the spot; a later submission whose beaten champion has since
been dethroned still lands its ghosts but the crown lapses (`crowned: false`).

## Test

`npm test` at the repo root runs the whole suite, server contract tests
included (in-process Hono app, in-memory SQLite, mock mailer — no network).
`npm test --workspace server` runs just these.
