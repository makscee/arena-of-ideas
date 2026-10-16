-- #1218 (mission #1214): log in with CrazyGames. A link is one CrazyGames
-- user (their userId from a verified token, never their username) and one
-- player, each at most once.
CREATE TABLE IF NOT EXISTS mvp_cg_links (
  cg_user_id TEXT PRIMARY KEY,
  player_id TEXT NOT NULL UNIQUE,
  json TEXT NOT NULL
);
