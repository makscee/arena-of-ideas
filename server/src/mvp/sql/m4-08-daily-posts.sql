-- Mission 4, M4-8 (makscee/void-board#816): the daily post to Telegram. One
-- row per day and message ("ru", "en" or a combined "ru+en"), claimed before
-- it is sent, so a retried day end or a restart never posts a day twice.
-- Only adds a table.
CREATE TABLE IF NOT EXISTS mvp_daily_posts (
  seq INTEGER NOT NULL,
  key TEXT NOT NULL,
  state TEXT NOT NULL,
  tries INTEGER NOT NULL,
  at TEXT NOT NULL,
  error TEXT,
  PRIMARY KEY (seq, key)
);
