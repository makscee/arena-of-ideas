# Mission 4: ready for players

Card: makscee/void-board#810. Plan page: https://m1.twin-pogona.ts.net/r/arena-m4-plan.html. Concept §10–11. Branch `mission-810-players`, cut from main at 8c0cac8e (missions 2 and 3 accepted). This file wins over the plan page where they differ.

Why: live has 3 human players and 0 ideas written. Ideas, votes and the daily trickle need people. This round brings them in: the game in Russian, a login that follows a player across devices, and a daily Telegram post that brings them back.

## Maks's answers (2026-10-08)
1. Players first (this mission). "The game looks after itself" (weekly balance pass, emergency fix, missing words) is mission 5.
2. **Telegram login: go** (auth: the orchestrator reviews and merges that slice).
3. The bot exists: **@arenaofideas_bot**. Its token is in `~/.config/arena/telegram.env` on m1 (`TELEGRAM_BOT_TOKEN=…`, mode 600). The channel's @name comes later.

## Secrets (all slices)
- The token is read at runtime from the file named by `ARENA_TELEGRAM_ENV` (the live plist sets it to `$HOME/.config/arena/telegram.env`). It never goes into the repo, the plist, logs, PRs, issues or screenshots. Tests use a fake Telegram (no network).
- Workers develop against a fake Telegram API (a local stub). Only the orchestrator talks to the real bot, after review.

## Calls (Maks can overturn any)
- **Language:** the phone's language (`navigator.language` starting `ru` → Russian, else English), then a switch in settings, kept on the device (and on the player, once logged in). Server-made text (refusals, captions, the daily post) is chosen per request by an `Accept-Language`-style header the client sends.
- **Card text** comes from the game's own words (`src/describe.ts`, `src/mvp/form-text.ts`, `src/glossary.ts`), so Russian is a second word table plus Russian grammar rules (case after "to": "атакующему", "всем врагам"), not translated sentences.
- **Unit names and lines** are translated once by Claude and stored **beside** the unit, never in its `Row` (`StoredUnit.texts = { ru: { name, line } }`), so the content version and every golden stay the same. Maks skims a one-page list before they go live. Bot names, player names and champion team names stay as written.
- **Fusion names:** the namer already names in English; a Russian form is added the same way (local model, the blocklist), stored beside the English.
- **Ideas:** the reader accepts either language and returns the name and line in both. Archetype words stay the game's own.
- **Telegram login is a deep link, not the web widget** (the widget needs a domain set in BotFather and is awkward on phones):
  1. "Войти через Telegram / Log in with Telegram" makes a one-time code and opens `https://t.me/arenaofideas_bot?start=<code>` (on a laptop, also a QR).
  2. The bot gets `/start <code>` (long polling `getUpdates`, one poller in the server), links that Telegram user to the player whose code it is (or, when that Telegram user is already linked, logs this device in as that player), and replies "Done, go back to the game". It never acts on `/start` alone: it first asks what the code would do (log a device in as @name, link @name, or create @name) with [Yes] [No] buttons, and only Yes acts, checked again at the tap (the same Telegram user, the code unused and in time, the links unchanged). No shows "Declined in Telegram" on the page (orchestrator's review of PR #706).
  3. The page polls the code and gets a session token (the same kind invite links give: `X-Arena-Token`).
  - Codes expire after 10 minutes and work once. A Telegram user links to one player; a player to one Telegram user. Unlinking is in settings.
  - Admin stays invite-only: a Telegram login never makes an admin.
- **The daily post:** after the day end (04:00 Moscow), one message per language to the channel (`ARENA_TELEGRAM_CHANNEL`): the new champion and their team, the number of slayers, units that entered and left (with "idea by @…"), and a share card image. Off unless `ARENA_TELEGRAM_POST=1`; `npm run mvp:post -- --dry-run` prints it. The orchestrator turns it on after Maks has seen a dry run.
- **Share cards:** PNGs made on the server from the game's own card look (an SVG template rendered to PNG), at `/share/champion/<day>.png` and `/share/unit/<id>.png`, each linking back to the game (`https://arena.makscee.ru/arena/`).
- Still out: the weekly balance pass, money, Discord, Steam.

## Slices
| # | Slice | Waits for | Risk |
|---|---|---|---|
| 1 (#811) | Strings in one place (English catalog) | – | |
| 2 (#814) | Russian screens + language switch | 1 | |
| 3 (#815) | Russian rules: card text, keywords, captions, traces | 1 | |
| 4 (#817) | Russian names: units and fusions | 3 | prod data |
| 5 (#818) | Ideas in Russian | 4 | |
| 6 (#812) | Log in with Telegram | – | auth |
| 7 (#813) | Share cards | – | |
| 8 (#816) | The daily post | 7 | outward |
| 9 | Bots, e2e both languages, playtest, report (orchestrator) | all | |

## Rules for every slice
- PR into `mission-810-players`, never main. Fresh clone: `git clone --depth 1 --single-branch --branch mission-810-players https://github.com/makscee/arena-of-ideas.git`.
- Migrations only add (`server/src/mvp/sql/m4-NN-*.sql`), and `pool.test.ts`'s list is updated.
- The content version must not change (units' rows untouched): check `npm run mvp:pool -- show` on a copy of a big DB.
- Never run a bot, e2e or anything that registers, plays or writes against the live instance (arena.makscee.ru, m1 /arena). Never call the real Telegram bot.
- The orchestrator (m1/void-board) checks the mission branch and redeploys live after merges, with a DB copy first.
