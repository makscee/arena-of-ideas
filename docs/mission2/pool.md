# Scout notes: the live pool, day change, votes (against 9ef07ac)

A line stores its own units, so most of the "content changed" machinery can relax. Only shop rolls and adding copies need the unit's current definition.

## How content versioning works now
- **The pool is code.** `ROWS` in `src/mvp/units.ts:254` is built by `mvpPool()` (`:542`).
  - The version is a hash of the whole pool (`server/src/mvp/content.ts:11`).
  - It is loaded once at startup (`main.ts:38`) and held as fixed `rt.content` (`runtime.ts:55`).
- **Everything checks `contentVersion === content.version`**, and a mismatch ends or skips:
  - runs end as "content-changed" (`runs.ts:59-60, 82-83, 124, 218`);
  - ghosts match only on the same version (`sqlite-store.ts:82`, `runs.ts:175`);
  - the Crown and the playoff skip other versions (`runs.ts:137, 159`, `src/mvp/day.ts:95, 112`);
  - stats are tallied per version (`stats.ts:44, 58`);
  - **bots replace the champion** whenever its version differs (`bots.ts:341, 346`).
- **Lines don't need the pool.** A `LineUnit` carries its own `recipe`, `stats`, `name` and `fusion` (`contract.ts:279`).
  - A fight reads only `abilities` and `statuses` from content (`fight.ts:72-73`), and abilities are built from their names (`units.ts:98`).
  - So an old line still fights as long as the ability registry only grows.
- **What still needs the unit's definition:** shop offers and gift draws (`run.ts:99, 112`), adding a copy (`forms.ts:70`), fusing (`runs.ts:251` `unitById`) and bot tier lookups (`bots.ts:133`).

## What must change
- Pool snapshots go in the DB. A run pins its version and resolves it through `poolFor(version)`.
- The ability and status registry becomes one union that only grows.
- Drop the version checks on ghosts, the Crown, the playoff and the champion seed. Any line whose ability names resolve can fight.
- Stats tally per unit per day.
- The namer's unit map is built at startup (`fusions.ts:535`) and must follow the pool.
- The phone fetches content once per page load (`mobile/content.ts:10`). It must refetch when `/health`'s `contentVersion` changes, and must resolve units that have left (champion lines, the Codex).
- The fusion total "6,480" is hardcoded (`codex.ts:11`).

## Data model (new `sql/` files, only CREATE TABLE, the store's JSON-per-row pattern, `store.ts:10-14`)
- `mvp_units`: unit_id, status `candidate|live|library|rejected`, row json in the `Row` shape, author_id, origin `idea|evolution|return|seed`, parent_id, created_at.
- `mvp_pools`: version, day_seq, unit_ids json, created_at.
- `mvp_pool_stints`: unit_id, entered_seq, left_seq, reason. A creator's number is the sum of live days over their units' stints.
- `mvp_ideas`: id, player_id, text, state, candidate unit_id, at. Credits go on the rating row's JSON, or in their own table.
- `mvp_votes`: id, voter_id, a_unit, b_unit, winner, at. Unique (voter, pair).
- `mvp_unit_day_tallies`: day_seq, unit_id, fights, wins, runs, picks.

## Day change
`endDay` (`server/src/mvp/day.ts:96`) runs in this order:
1. The playoff, on the pool that is ending.
2. Store the champion.
3. **Then** rotation: write the snapshot and the stints.
4. `putDay(next)` last.

Every write is keyed by seq and idempotent before `putDay` (`:118-121`), so a retry is safe. `rt.content` becomes a getter on the current pool. The bot job (`bots.ts:380`) refills thin ghost rounds on its own once matching ignores the version.

## Risks
- Migrations: the runner applies each file once in its own transaction (`sqlite-store.ts:20-33`). Copy the DB file before deploy.
- The first deploy with a new pool hash ends every active run unless pinning ships first.
- Units are keyed by `slug(name)` (`units.ts:391`). Ids must be unique and never reused.
- Player text goes through `crude.ts`. Every idea is validated with `formProblems` (`forms.ts:147`).
