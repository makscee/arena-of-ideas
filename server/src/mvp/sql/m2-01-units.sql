-- M2-1 (mission #735): units as data. The pool's source of truth moves from
-- code (src/mvp/units.ts ROWS) into these tables; ./pool.ts seeds them once
-- and builds the content from the current pool. Each row keeps the whole
-- object as JSON; the other columns are only what the store looks rows up by.
-- A unit's id is permanent and never reused; its row is the `Row` shape.
CREATE TABLE IF NOT EXISTS mvp_units (unit_id TEXT PRIMARY KEY, status TEXT NOT NULL, author_id TEXT, json TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS mvp_units_status ON mvp_units (status);
-- Pool snapshots: the live units at a day, newest last (seq). `version` is the
-- content version built from them; the same set gives the same version.
CREATE TABLE IF NOT EXISTS mvp_pools (seq INTEGER PRIMARY KEY AUTOINCREMENT, version TEXT NOT NULL, day_seq INTEGER NOT NULL, json TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS mvp_pools_version ON mvp_pools (version);
-- A unit's stays in the live pool: entered at a day, left at one (or not yet).
CREATE TABLE IF NOT EXISTS mvp_pool_stints (unit_id TEXT NOT NULL, entered_seq INTEGER NOT NULL, json TEXT NOT NULL, PRIMARY KEY (unit_id, entered_seq));
-- Unit tallies per day (./stats.ts), beside the per-version ones (11-stats.sql):
-- finished runs per day, and per unit the fights its team fought and won, the
-- finished runs it ended on the line in and the times it was picked (bought or
-- taken as a gift).
CREATE TABLE IF NOT EXISTS mvp_day_run_tallies (day_seq INTEGER PRIMARY KEY, runs INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS mvp_unit_day_tallies (
  day_seq INTEGER NOT NULL,
  unit_id TEXT NOT NULL,
  fights INTEGER NOT NULL,
  wins INTEGER NOT NULL,
  runs INTEGER NOT NULL,
  picks INTEGER NOT NULL,
  PRIMARY KEY (day_seq, unit_id)
);
