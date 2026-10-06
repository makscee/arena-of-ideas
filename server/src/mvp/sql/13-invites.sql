-- Slice 13 (mission #574): invite links. An invite is one person's link: its
-- code (the secret in the URL) names one player, and opening it on any device
-- starts a session for that player. Sessions keep only the token's SHA-256.
CREATE TABLE IF NOT EXISTS mvp_invites (
  code TEXT PRIMARY KEY,
  name_key TEXT NOT NULL UNIQUE,
  player_id TEXT NOT NULL,
  json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS mvp_sessions (
  token_hash TEXT PRIMARY KEY,
  player_id TEXT NOT NULL,
  created_at TEXT NOT NULL
);
