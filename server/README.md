# Arena server

Standalone server for arena-of-ideas: email-OTP accounts now, the shared
ladder API in later slices. Hono + Drizzle + better-sqlite3, run with tsx,
tested with vitest — same toolchain as the kernel.

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

## Test

`npm test` at the repo root runs the whole suite, server contract tests
included (in-process Hono app, in-memory SQLite, mock mailer — no network).
`npm test --workspace server` runs just these.
