import { and, asc, eq } from "drizzle-orm";
import { isDeepStrictEqual } from "node:util";
import {
  DEFAULT_SELECTION_TUNABLES,
  mergePool,
  selectSeason,
  talliesOf,
  type ApprovedRegistry,
  type LadderData,
  type UnitDef,
} from "../../src/index.js";
import { approveInto } from "../../src/create/approve.js";
import { parseCandidateRecord } from "../../src/create/candidates.js";
import type { CandidateRecord } from "../../src/create/provenance.js";
import type Database from "better-sqlite3";
import type { DB } from "./db.js";
import { listIdeas } from "./ideas.js";
import { readActiveContent, readSeasonPointer } from "./season-store.js";
import {
  contentVersions,
  ideaBuilds,
  ideas,
  ideaVotes,
  ladderChampions,
  ladderGhosts,
  runOpens,
  runPoolServes,
  seasonArchives,
  seasonFreezes,
  seasonState,
  users,
} from "./schema.js";

export interface GovernanceDeps { db: DB; sqlite: Database.Database; clock: () => number }
export interface VersionExpectation { season: number; contentVersion: number }

export class GovernanceError extends Error {
  constructor(message: string) { super(message); this.name = "GovernanceError"; }
}

function assertPointer(db: DB, expected: VersionExpectation): void {
  const actual = readSeasonPointer(db);
  if (actual.season !== expected.season || actual.contentVersion !== expected.contentVersion) {
    throw new GovernanceError(`stale season/content input: expected season ${expected.season} / content v${expected.contentVersion}, active is season ${actual.season} / content v${actual.contentVersion}`);
  }
}

function selectedRows(db: DB, season: number) {
  return db.select().from(ideaBuilds).where(eq(ideaBuilds.season, season)).orderBy(asc(ideaBuilds.rank)).all();
}

function selectionReceiptOf(rows: ReturnType<typeof selectedRows>) {
  return {
    tunables: DEFAULT_SELECTION_TUNABLES,
    selected: rows.map((row) => ({ ideaId: row.ideaId, rank: row.rank, tally: JSON.parse(row.tally) })),
  };
}

function requireFrozenReceipt(deps: GovernanceDeps, expected: VersionExpectation, rows: ReturnType<typeof selectedRows>) {
  const frozen = deps.db.select().from(seasonFreezes).where(eq(seasonFreezes.season, expected.season)).all()[0];
  if (frozen === undefined) throw new GovernanceError(`season ${expected.season} selection is not frozen`);
  if (frozen.contentVersion !== expected.contentVersion) {
    throw new GovernanceError(`season ${expected.season} freeze receipt is for content v${frozen.contentVersion}, not v${expected.contentVersion}`);
  }
  let receipt: unknown;
  try { receipt = JSON.parse(frozen.selectionReceipt); }
  catch (err) { throw new GovernanceError(`season ${expected.season} freeze receipt is corrupt JSON: ${(err as Error).message}`); }
  if (!isDeepStrictEqual(receipt, selectionReceiptOf(rows))) {
    throw new GovernanceError(`season ${expected.season} build slate does not match its frozen selection receipt`);
  }
  return frozen;
}

export function freezeSelection(deps: GovernanceDeps, expected: VersionExpectation) {
  let selection: ReturnType<typeof selectSeason> | undefined;
  const tx = deps.sqlite.transaction(() => {
    assertPointer(deps.db, expected);
    if (deps.db.select().from(seasonFreezes).where(eq(seasonFreezes.season, expected.season)).all().length > 0) {
      throw new GovernanceError(`season ${expected.season} selection is already frozen`);
    }
    const table = listIdeas({ db: deps.db, clock: deps.clock }).filter((idea) => idea.status !== "shipped" && idea.status !== "selected");
    selection = selectSeason(table, talliesOf(table), DEFAULT_SELECTION_TUNABLES);
    const receipt = {
      tunables: DEFAULT_SELECTION_TUNABLES,
      selected: selection.selected.map((ranked, rank) => ({ ideaId: ranked.idea.id, rank, tally: ranked.tally })),
    };
    deps.db.insert(seasonFreezes).values({
      season: expected.season,
      contentVersion: expected.contentVersion,
      selectionReceipt: JSON.stringify(receipt),
      frozenAt: deps.clock(),
    }).run();
    selection.selected.forEach((ranked, rank) => {
      deps.db.insert(ideaBuilds).values({
        season: expected.season,
        ideaId: ranked.idea.id,
        rank,
        tally: JSON.stringify(ranked.tally),
        status: "building",
        updatedAt: deps.clock(),
      }).run();
      deps.db.update(ideas).set({ status: "selected", bounceReason: null }).where(eq(ideas.id, ranked.idea.id)).run();
    });
  });
  tx.immediate();
  return { season: expected.season, contentVersion: expected.contentVersion, tunables: DEFAULT_SELECTION_TUNABLES, selection: selection! };
}

function requireBuilding(deps: GovernanceDeps, expected: VersionExpectation, ideaId: string) {
  assertPointer(deps.db, expected);
  const rows = selectedRows(deps.db, expected.season);
  requireFrozenReceipt(deps, expected, rows);
  const row = rows.find((candidate) => candidate.ideaId === ideaId);
  if (row === undefined) throw new GovernanceError(`idea ${ideaId} is not selected in frozen season ${expected.season}`);
  if (row.status !== "building") throw new GovernanceError(`idea ${ideaId} already has outcome ${row.status}; duplicate staging is not allowed`);
  return row;
}

function candidateWithCreator(candidate: CandidateRecord, displayName: string): CandidateRecord {
  return { ...candidate, provenance: { ...candidate.provenance, creator: displayName } };
}

function basePool(active: ReturnType<typeof readActiveContent>): UnitDef[] {
  const approved = new Set(active.approvedRegistry.units.map((unit) => unit.name));
  return active.pool.filter((unit) => !approved.has(unit.name));
}

export function stageShip(
  deps: GovernanceDeps,
  expected: VersionExpectation,
  ideaId: string,
  candidateData: unknown,
  candidateLabel: string,
) {
  requireBuilding(deps, expected, ideaId);
  const active = readActiveContent(deps.db);
  const parsed = parseCandidateRecord(candidateData, active.statuses, active.abilities, candidateLabel);
  const idea = deps.db.select().from(ideas).where(eq(ideas.id, ideaId)).all()[0];
  if (idea === undefined) throw new GovernanceError(`selected idea ${ideaId} is missing`);
  const author = deps.db.select().from(users).where(eq(users.id, idea.authorId)).all()[0];
  if (author === undefined) throw new GovernanceError(`idea ${ideaId} author ${idea.authorId} is missing`);
  const displayName = author.displayName?.trim();
  if (!displayName) throw new GovernanceError(`idea ${ideaId} author ${author.id} has no display name; set one before shipping`);
  const authoritative = candidateWithCreator(parsed, displayName);
  const approved = approveInto(
    active.approvedRegistry,
    authoritative,
    basePool(active).map((unit) => unit.name),
    active.statuses,
    active.abilities,
  );
  const added = approved.units.slice(active.approvedRegistry.units.length);
  const stagedNames = selectedRows(deps.db, expected.season)
    .filter((row) => row.status === "staged-ship")
    .flatMap((row) => row.shippedUnits ? (JSON.parse(row.shippedUnits) as UnitDef[]).map((unit) => unit.name) : []);
  const collision = added.find((unit) => stagedNames.includes(unit.name));
  if (collision) throw new GovernanceError(`candidate ${parsed.id} unit "${collision.name}" collides with another staged shipment`);

  deps.db.transaction(() => {
    requireBuilding(deps, expected, ideaId);
    deps.db.update(ideaBuilds).set({
      status: "staged-ship",
      candidate: JSON.stringify(authoritative),
      candidateProvenance: JSON.stringify(parsed.provenance),
      authorUserId: author.id,
      creatorDisplayName: displayName,
      shippedUnits: JSON.stringify(added),
      updatedAt: deps.clock(),
    }).where(and(eq(ideaBuilds.season, expected.season), eq(ideaBuilds.ideaId, ideaId))).run();
  });
  return { ideaId, candidateId: parsed.id, authorUserId: author.id, creatorDisplayName: displayName, units: added.map((unit) => unit.name), verdict: "re-sim passed" };
}

export function stageBounce(deps: GovernanceDeps, expected: VersionExpectation, ideaId: string, reason: string) {
  requireBuilding(deps, expected, ideaId);
  const clean = reason.trim();
  if (!clean) throw new GovernanceError("a bounced selected idea requires a non-empty reason");
  deps.db.transaction(() => {
    requireBuilding(deps, expected, ideaId);
    deps.db.update(ideaBuilds).set({ status: "staged-bounce", bounceReason: clean, updatedAt: deps.clock() })
      .where(and(eq(ideaBuilds.season, expected.season), eq(ideaBuilds.ideaId, ideaId))).run();
  });
  return { ideaId, reason: clean };
}

function towerSnapshot(db: DB): LadderData {
  const pools: LadderData["pools"] = {};
  for (const row of db.select().from(ladderGhosts).orderBy(asc(ladderGhosts.round), asc(ladderGhosts.seq)).all()) {
    const key = String(row.round);
    const list = pools[key] ?? [];
    list.push({ runId: row.runId, round: row.round, seq: row.seq, team: JSON.parse(row.team) as UnitDef[] });
    pools[key] = list;
  }
  const bosses: LadderData["bosses"] = {};
  const champions = db.select().from(ladderChampions).orderBy(asc(ladderChampions.id)).all();
  const current = champions.at(-1);
  if (current) bosses[String(current.round)] = { runId: current.runId, round: current.round, seq: current.seq, team: JSON.parse(current.team) as UnitDef[] };
  return { bosses, pools };
}

function compileRegistry(deps: GovernanceDeps, expected: VersionExpectation, rows: ReturnType<typeof selectedRows>): ApprovedRegistry {
  const active = readActiveContent(deps.db);
  const shippedNames = basePool(active).map((unit) => unit.name);
  let registry = active.approvedRegistry;
  for (const row of rows) {
    if (row.status !== "staged-ship") continue;
    if (!row.candidate) throw new GovernanceError(`idea ${row.ideaId} staged ship has no candidate receipt`);
    const candidate = parseCandidateRecord(JSON.parse(row.candidate), active.statuses, { ...active.abilities, ...(registry.abilities ?? {}) }, `staged candidate for ${row.ideaId}`);
    registry = approveInto(registry, candidate, shippedNames, active.statuses, { ...active.abilities, ...(registry.abilities ?? {}) });
    const added = registry.units.filter((unit) => !active.approvedRegistry.units.some((old) => old.name === unit.name));
    const recorded = row.shippedUnits ? JSON.parse(row.shippedUnits) as UnitDef[] : [];
    const own = recorded.map((unit) => unit.name);
    if (!own.every((name) => added.some((unit) => unit.name === name))) throw new GovernanceError(`idea ${row.ideaId} staged shipment receipt does not match compiled registry`);
  }
  return registry;
}

export function rollSeason(deps: GovernanceDeps, expected: VersionExpectation) {
  const tx = deps.sqlite.transaction(() => {
    // BEGIN IMMEDIATE is already held when this callback starts. Every fact that
    // defines the boundary is read, validated, compiled and written under it.
    assertPointer(deps.db, expected);
    const rows = selectedRows(deps.db, expected.season);
    const frozen = requireFrozenReceipt(deps, expected, rows);
    const unfinished = rows.find((row) => row.status !== "staged-ship" && row.status !== "staged-bounce");
    if (unfinished) throw new GovernanceError(`idea ${unfinished.ideaId} is still building; every selected idea needs a staged ship or bounce before roll`);
    if (deps.db.select().from(seasonArchives).where(eq(seasonArchives.season, expected.season)).all().length > 0) {
      throw new GovernanceError(`season ${expected.season} is already archived`);
    }
    const nextVersion = expected.contentVersion + 1;
    if (deps.db.select().from(contentVersions).where(eq(contentVersions.version, nextVersion)).all().length > 0) {
      throw new GovernanceError(`content version ${nextVersion} already exists`);
    }

    // Candidate parse, re-sim, collision validation and every receipt snapshot
    // stay under the same write lock as the tower snapshot and reset.
    const active = readActiveContent(deps.db);
    const registry = compileRegistry(deps, expected, rows);
    const pool = mergePool(basePool(active), registry.units);
    const abilities = { ...active.abilities, ...(registry.abilities ?? {}) };
    const finalTower = towerSnapshot(deps.db);
    const outcomeReceipt = rows.map((row) => ({
      ideaId: row.ideaId,
      outcome: row.status === "staged-ship" ? "shipped" : "bounced",
      ...(row.status === "staged-ship" ? {
        authorUserId: row.authorUserId,
        creatorDisplayName: row.creatorDisplayName,
        candidateProvenance: row.candidateProvenance ? JSON.parse(row.candidateProvenance) : null,
        units: row.shippedUnits ? (JSON.parse(row.shippedUnits) as UnitDef[]).map((unit) => unit.name) : [],
      } : { reason: row.bounceReason }),
    }));

    deps.sqlite.prepare("INSERT INTO season_archives (season, content_version, final_tower_json, selection_receipt_json, outcome_receipt_json, archived_at) VALUES (?, ?, ?, ?, ?, ?)")
      .run(expected.season, expected.contentVersion, JSON.stringify(finalTower), frozen.selectionReceipt, JSON.stringify(outcomeReceipt), deps.clock());
    deps.sqlite.prepare("INSERT INTO content_versions (version, approved_registry, pool, statuses, abilities, created_at) VALUES (?, ?, ?, ?, ?, ?)")
      .run(nextVersion, JSON.stringify(registry), JSON.stringify(pool), JSON.stringify(active.statuses), JSON.stringify(abilities), deps.clock());
    for (const row of rows) {
      const shipped = row.status === "staged-ship";
      deps.sqlite.prepare("UPDATE ideas SET status=?, bounce_reason=? WHERE id=?")
        .run(shipped ? "shipped" : "bounced", shipped ? null : row.bounceReason, row.ideaId);
      deps.sqlite.prepare("UPDATE idea_builds SET status=?, updated_at=? WHERE season=? AND idea_id=?")
        .run(shipped ? "shipped" : "bounced", deps.clock(), expected.season, row.ideaId);
    }
    // Production roll semantics: the new season starts with an empty tower.
    // Old opens stay as immutable provenance but are unusable because their
    // content_version no longer matches; every served view is invalidated.
    deps.sqlite.exec("DELETE FROM run_pool_serves; DELETE FROM ladder_champions; DELETE FROM ladder_ghosts;");
    deps.sqlite.prepare("UPDATE season_state SET season=?, content_version=? WHERE id=1")
      .run(expected.season + 1, nextVersion);
    return {
      archivedSeason: expected.season,
      archivedContentVersion: expected.contentVersion,
      season: expected.season + 1,
      contentVersion: nextVersion,
      shipped: outcomeReceipt.filter((receipt) => receipt.outcome === "shipped"),
      bounced: outcomeReceipt.filter((receipt) => receipt.outcome === "bounced"),
    };
  });
  return tx.immediate();
}

/** E2E-only deterministic fixture. It is intentionally impossible to invoke
 * without the harness flag; production governance has no reset operation. */
export function resetAoi62Fixture(deps: GovernanceDeps, enabled: boolean) {
  if (!enabled) throw new GovernanceError("fixture reset is e2e-only; set AOI_E2E=1 on a temporary DB");
  const v1 = deps.db.select().from(contentVersions).where(eq(contentVersions.version, 1)).all()[0];
  if (!v1) throw new GovernanceError("fixture reset requires seeded content v1");
  const unit = (JSON.parse(v1.pool) as UnitDef[])[0]!;
  const tx = deps.sqlite.transaction(() => {
    deps.sqlite.exec(`
      DELETE FROM run_pool_serves; DELETE FROM run_opens; DELETE FROM run_submissions;
      DELETE FROM ladder_champions; DELETE FROM ladder_ghosts;
      DELETE FROM idea_votes; DELETE FROM idea_builds; DELETE FROM season_freezes; DELETE FROM ideas;
      DELETE FROM season_archives; DELETE FROM sessions; DELETE FROM email_codes; DELETE FROM users;
      DELETE FROM content_versions WHERE version <> 1;
      UPDATE season_state SET season=1, content_version=1 WHERE id=1;
    `);
    const people = [
      ["fixture-maks", "maks@aoi62.test", "Maks"],
      ["fixture-glass", "glass@aoi62.test", "Glass Smith"],
      ...Array.from({ length: 6 }, (_, i) => [`fixture-voter-${i + 1}`, `voter${i + 1}@aoi62.test`, `Voter ${i + 1}`]),
    ];
    for (const [id, email, name] of people) deps.sqlite.prepare("INSERT INTO users (id,email,display_name,created_at,updated_at) VALUES (?,?,?,?,?)").run(id, email, name, deps.clock(), deps.clock());
    const team = JSON.stringify([unit]);
    deps.sqlite.prepare("INSERT INTO ladder_ghosts (round,seq,run_id,user_id,team) VALUES (1,0,'fixture-season-1','fixture-maks',?)").run(team);
    deps.sqlite.prepare("INSERT INTO ladder_champions (run_id,user_id,round,seq,team) VALUES ('fixture-season-1','fixture-maks',1,0,?)").run(team);
  });
  tx.immediate();
  return { fixture: "aoi62-governance", scenarios: ["ship-frostbiter-by-maks", "bounce-glass-cannon"], season: 1, contentVersion: 1, authorEmail: "maks@aoi62.test", bounceAuthorEmail: "glass@aoi62.test" };
}

export function castAoi62FixtureVotes(deps: GovernanceDeps, enabled: boolean, shipIdeaId: string, bounceIdeaId: string) {
  if (!enabled) throw new GovernanceError("fixture votes are e2e-only; set AOI_E2E=1 on a temporary DB");
  for (const id of [shipIdeaId, bounceIdeaId]) if (!deps.db.select().from(ideas).where(eq(ideas.id, id)).all()[0]) throw new GovernanceError(`fixture idea ${id} is missing`);
  deps.db.transaction(() => {
    for (let i = 1; i <= 5; i++) {
      deps.db.insert(ideaVotes).values({ ideaId: shipIdeaId, userId: `fixture-voter-${i}`, direction: "up", votedAt: deps.clock() }).run();
      deps.db.insert(ideaVotes).values({ ideaId: bounceIdeaId, userId: `fixture-voter-${i}`, direction: i === 5 ? "down" : "up", votedAt: deps.clock() }).run();
    }
  });
  return { ship: { scenario: "ship-frostbiter-by-maks", ideaId: shipIdeaId, up: 5, total: 5 }, bounce: { scenario: "bounce-glass-cannon", ideaId: bounceIdeaId, up: 4, total: 5 } };
}
