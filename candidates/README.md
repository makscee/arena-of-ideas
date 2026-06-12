# Candidates pool (PRD #013 slice 4)

Passing candidates from the creation loop land here — one JSON file per
candidate, `<id>.json` — each carrying the candidate's units plus full
**provenance**: the idea text, who gets credit, the harness/model that produced
it, the gauntlet numbers it passed on (pooled win-rate + every matchup), the
run's timestamp, and how many bounce attempts it took. The pool is append-only
and legible; every record is validated on read (the same content gate a battle
input passes).

## The flow

1. **Create** — `npm run create -- tasks/<id>` drives a harness until it emits an
   in-band candidate (`tasks/<id>/out/candidate.json`) and a bounce log
   (`out/run-log.jsonl`).
2. **Mint** — `npm run mint-candidate -- tasks/<id> --creator <name>
   [--harness h] [--model m]` reads the log's gate numbers, the idea text, and
   the run's timestamp, and writes the provenance record to `candidates/<id>.json`.
3. **Approve** — `npm run approve` lists the pending candidates; `npm run approve
   -- <id>` moves the candidate's NEW units into the playable registry
   (`registry/approved-units.json`). The web run screen merges that registry onto
   the shipped pool, so a new run can draft the unit and the codex catalogues it
   with a "made by …" credit line.

Approval is bookkeeping, not judgement — the gauntlet already decided the unit is
in-band. Shipped support units in a candidate team (e.g. Squire) are skipped on
approve; only the genuinely new creation is added. A name collision with a
shipped or already-approved unit is refused loudly.
