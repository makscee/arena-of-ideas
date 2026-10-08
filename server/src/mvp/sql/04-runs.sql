-- Slice 4 (mission #574): the MVP's record. Each row keeps the whole object
-- as JSON; the other columns are only what the store looks rows up by.
CREATE TABLE IF NOT EXISTS mvp_players (id TEXT PRIMARY KEY, json TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS mvp_runs (id TEXT PRIMARY KEY, player_id TEXT NOT NULL, over INTEGER NOT NULL, json TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS mvp_runs_active ON mvp_runs (player_id, over);
CREATE TABLE IF NOT EXISTS mvp_ghosts (seq INTEGER PRIMARY KEY AUTOINCREMENT, ghost_id TEXT NOT NULL, round INTEGER NOT NULL, player_id TEXT NOT NULL, content_version TEXT NOT NULL, json TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS mvp_ghosts_round ON mvp_ghosts (round, content_version);
CREATE TABLE IF NOT EXISTS mvp_battles (id TEXT PRIMARY KEY, kind TEXT NOT NULL, at TEXT NOT NULL, json TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS mvp_battles_at ON mvp_battles (at);
CREATE TABLE IF NOT EXISTS mvp_fusions (first TEXT NOT NULL, second TEXT NOT NULL, json TEXT NOT NULL, PRIMARY KEY (first, second));
CREATE TABLE IF NOT EXISTS mvp_days (seq INTEGER PRIMARY KEY, json TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS mvp_champions (seq INTEGER PRIMARY KEY, json TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS mvp_slays (id INTEGER PRIMARY KEY AUTOINCREMENT, seq INTEGER NOT NULL, json TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS mvp_slays_seq ON mvp_slays (seq);
CREATE TABLE IF NOT EXISTS mvp_playoffs (seq INTEGER PRIMARY KEY, json TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS mvp_ratings (player_id TEXT PRIMARY KEY, json TEXT NOT NULL);
