-- Mission 2, M2-8 (makscee/void-board#793): players' either/or votes between a
-- candidate unit (mvp_units status 'candidate') and a typical live unit.
-- `pick` is the unit chosen, or NULL for a skip. One vote per player per
-- pair (the primary key). Only adds a table.
CREATE TABLE IF NOT EXISTS mvp_votes (
  player_id TEXT NOT NULL,
  candidate_id TEXT NOT NULL,
  other_id TEXT NOT NULL,
  pick TEXT,
  created_at TEXT NOT NULL,
  PRIMARY KEY (player_id, candidate_id, other_id)
);
CREATE INDEX IF NOT EXISTS mvp_votes_candidate ON mvp_votes (candidate_id);
