-- M2-2 (mission #735): a pool change ends nothing. Ghosts are picked from any
-- pool, so the pick reads a round's newest ghosts by seq.
CREATE INDEX IF NOT EXISTS mvp_ghosts_round_seq ON mvp_ghosts (round, seq);
