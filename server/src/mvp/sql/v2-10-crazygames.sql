-- #1218 (mission #1214): log in with CrazyGames. A link is one CrazyGames
-- user (their userId from a verified token, never their username) and one
-- player, each at most once.
CREATE TABLE IF NOT EXISTS mvp_cg_links (
  cg_user_id TEXT PRIMARY KEY,
  player_id TEXT NOT NULL UNIQUE,
  json TEXT NOT NULL
);

-- The players a guest join made: only these may take a CrazyGames link, and
-- they play without the email an invite-only server asks of others.
CREATE TABLE IF NOT EXISTS mvp_guests (
  player_id TEXT PRIMARY KEY,
  created_at TEXT NOT NULL
);
