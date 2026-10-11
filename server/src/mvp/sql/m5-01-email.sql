-- #929: email login. Every account is tied to one email (trimmed, lower-cased:
-- email_key), each side at most once. A login code is 6 digits, kept only as
-- its SHA-256, one live code per email (a new one replaces it), 10 minutes,
-- 5 tries.
CREATE TABLE IF NOT EXISTS mvp_emails (
  email_key TEXT PRIMARY KEY,
  player_id TEXT NOT NULL UNIQUE,
  json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS mvp_email_codes (
  email_key TEXT PRIMARY KEY,
  json TEXT NOT NULL
);
