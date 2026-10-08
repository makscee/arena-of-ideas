-- M2-2 (mission #735): GET /stats reads only the per-day tallies now. A
-- database from before M2-1 holds its history in the per-version tallies
-- (11-stats.sql) alone, so this puts that history on the current day, once:
-- what the per-version tallies counted minus what the day tallies already
-- counted (both are written by the same hooks since M2-1). A database
-- without a day yet gets nothing.
CREATE TEMP TABLE m2_backfill_unit AS
  SELECT v.unit_id AS unit_id,
         MAX(0, v.fights - COALESCE(d.fights, 0)) AS fights,
         MAX(0, v.wins - COALESCE(d.wins, 0)) AS wins,
         MAX(0, v.runs - COALESCE(d.runs, 0)) AS runs
  FROM (SELECT unit_id, SUM(fights) AS fights, SUM(wins) AS wins, SUM(runs) AS runs FROM mvp_unit_tallies GROUP BY unit_id) v
  LEFT JOIN (SELECT unit_id, SUM(fights) AS fights, SUM(wins) AS wins, SUM(runs) AS runs FROM mvp_unit_day_tallies GROUP BY unit_id) d
    ON d.unit_id = v.unit_id;

INSERT INTO mvp_unit_day_tallies (day_seq, unit_id, fights, wins, runs, picks)
  SELECT (SELECT MAX(seq) FROM mvp_days), unit_id, fights, wins, runs, 0
  FROM m2_backfill_unit
  WHERE (SELECT MAX(seq) FROM mvp_days) IS NOT NULL AND (fights > 0 OR runs > 0)
  ON CONFLICT (day_seq, unit_id) DO UPDATE SET
    fights = fights + excluded.fights, wins = wins + excluded.wins, runs = runs + excluded.runs;

INSERT INTO mvp_day_run_tallies (day_seq, runs)
  SELECT (SELECT MAX(seq) FROM mvp_days),
         MAX(0, (SELECT COALESCE(SUM(runs), 0) FROM mvp_run_tallies) - (SELECT COALESCE(SUM(runs), 0) FROM mvp_day_run_tallies))
  WHERE (SELECT MAX(seq) FROM mvp_days) IS NOT NULL
  ON CONFLICT (day_seq) DO UPDATE SET runs = runs + excluded.runs;

DROP TABLE m2_backfill_unit;
