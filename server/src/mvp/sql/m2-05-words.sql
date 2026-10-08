-- Mission 2, M2-5 (makscee/void-board#792): the new-words log. An idea the
-- reader couldn't fully make names the game word it lacks ("steal gold") and
-- the part of its text that needed it (WordRequest in ../store.ts). Only
-- additions, never rewritten. Only adds a table.
CREATE TABLE IF NOT EXISTS mvp_word_requests (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  idea_id TEXT NOT NULL,
  word TEXT NOT NULL,
  part TEXT NOT NULL,
  created_at TEXT NOT NULL
);
