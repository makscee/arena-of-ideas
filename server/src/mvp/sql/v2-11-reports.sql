-- #1271 (mission #1214): a player reports another player's name or a
-- player-made unit. One report per reporter and target; nothing is hidden
-- by it, the admin reads them (npm run mvp:reports).
CREATE TABLE IF NOT EXISTS mvp_reports (
  id TEXT PRIMARY KEY,
  reporter_id TEXT NOT NULL,
  kind TEXT NOT NULL,
  target_id TEXT NOT NULL,
  reason TEXT,
  created_at TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'open',
  UNIQUE (reporter_id, kind, target_id)
);
CREATE INDEX IF NOT EXISTS mvp_reports_by ON mvp_reports (reporter_id, created_at);
