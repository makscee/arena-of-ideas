-- Rules v2, word forge (makscee/void-board#1052): one request queue for every
-- source of a new word. mvp_word_requests (m2-05, idea readings only) gains
-- the source ("idea", "night", "batch") and what asked (an idea id, a pair
-- key, a unit id); idea_id stays as the old column and mirrors ref. A word is
-- found by the first claimed design that uses it: one row per word, the first
-- claim wins. Only additions.
ALTER TABLE mvp_word_requests ADD COLUMN source TEXT NOT NULL DEFAULT 'idea';
ALTER TABLE mvp_word_requests ADD COLUMN ref TEXT NOT NULL DEFAULT '';
UPDATE mvp_word_requests SET ref = idea_id;
CREATE TABLE IF NOT EXISTS mvp_words_found (
  key TEXT PRIMARY KEY,
  word TEXT NOT NULL,
  kind TEXT NOT NULL,
  pair TEXT NOT NULL,
  player_id TEXT NOT NULL,
  found_at TEXT NOT NULL
);
