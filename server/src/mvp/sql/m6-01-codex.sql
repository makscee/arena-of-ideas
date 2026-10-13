-- Rules v2, the Codex (makscee/void-board#1049, mission #1043): one row per
-- unordered pair (a < b) with the night batch's candidates and the claim
-- (CodexPair in src/mvp/codex.ts), and players' wishes, one per player, pair
-- and day. Only adds tables.
CREATE TABLE IF NOT EXISTS mvp_codex_pairs (
  a TEXT NOT NULL,
  b TEXT NOT NULL,
  claimed INTEGER NOT NULL DEFAULT 0,
  json TEXT NOT NULL,
  PRIMARY KEY (a, b)
);
CREATE TABLE IF NOT EXISTS mvp_codex_wishes (
  a TEXT NOT NULL,
  b TEXT NOT NULL,
  player_id TEXT NOT NULL,
  seq INTEGER NOT NULL,
  at TEXT NOT NULL,
  PRIMARY KEY (a, b, player_id, seq)
);
CREATE INDEX IF NOT EXISTS mvp_codex_wishes_seq ON mvp_codex_wishes (seq);
