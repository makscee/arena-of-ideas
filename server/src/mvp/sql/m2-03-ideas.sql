-- Mission 2, M2-3 (makscee/void-board#787): what a player's ideas are counted
-- from besides their finished runs (mvp_ratings): spent, dev-granted and
-- forfeited counts (IdeaCounts in ../store.ts). Only adds a table.
CREATE TABLE IF NOT EXISTS mvp_idea_counts (
  player_id TEXT PRIMARY KEY,
  json TEXT NOT NULL
);
