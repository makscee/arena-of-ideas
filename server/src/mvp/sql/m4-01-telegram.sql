-- M4-6 (mission #810): log in with Telegram. A link is one Telegram user and
-- one player, each at most once. A login code is a one-time deep-link code
-- (t.me/<bot>?start=<code>), kept only as its SHA-256, for 10 minutes.
CREATE TABLE IF NOT EXISTS mvp_telegram_links (
  tg_user_id TEXT PRIMARY KEY,
  player_id TEXT NOT NULL UNIQUE,
  json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS mvp_telegram_codes (
  code_hash TEXT PRIMARY KEY,
  json TEXT NOT NULL
);
