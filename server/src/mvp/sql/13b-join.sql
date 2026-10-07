-- R4-20 (mission #574, #704): the open join link. One shared code (the secret
-- in …/arena/#join=<code>) lets anyone pick a name and become a new player.
-- Kept as a setting: key 'join_code'.
CREATE TABLE IF NOT EXISTS mvp_settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
