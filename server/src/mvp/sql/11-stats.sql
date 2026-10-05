-- Slice 11 (mission #574): unit win and pick rates, counted as runs go
-- (./stats.ts), per content version. Finished runs per version, and per unit
-- the fights its team fought and won and the finished runs it ended in.
CREATE TABLE IF NOT EXISTS mvp_run_tallies (content_version TEXT PRIMARY KEY, runs INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS mvp_unit_tallies (
  content_version TEXT NOT NULL,
  unit_id TEXT NOT NULL,
  fights INTEGER NOT NULL,
  wins INTEGER NOT NULL,
  runs INTEGER NOT NULL,
  PRIMARY KEY (content_version, unit_id)
);
