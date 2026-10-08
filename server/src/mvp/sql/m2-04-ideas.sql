-- Mission 2, M2-4 (makscee/void-board#790): written ideas (Idea in
-- src/mvp/contract.ts). `text` is private to its author; `json` holds what
-- later stages add (Idea.data). Only adds a table.
CREATE TABLE IF NOT EXISTS mvp_ideas (
  id TEXT PRIMARY KEY,
  player_id TEXT NOT NULL,
  text TEXT NOT NULL,
  state TEXT NOT NULL,
  created_at TEXT NOT NULL,
  json TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS mvp_ideas_player ON mvp_ideas (player_id);
CREATE INDEX IF NOT EXISTS mvp_ideas_state ON mvp_ideas (state);
