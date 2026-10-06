// MVP storage seam (mission #574). Tests keep everything in memory behind
// this interface; main.ts passes slice 4's SQLite store (./sqlite-store.ts)
// to mvpRuntime, so the routes and the run engine never change.
//
// Rules for every slice that touches storage (several build in parallel):
// - Slice 4 implements every method that exists when it merges, fusions
//   included: slice 10 owns what is written there, not the table.
// - A slice that adds a method implements it in every store class and adds
//   its case to describeMvpStore (./store.contract.ts), which every class runs.
// - SQL goes in server/src/mvp/sql/<slice>-<name>.sql, the slice two digits
//   (04-runs.sql, 05-days.sql), applied in filename order at startup, each
//   once: the applied names are recorded in mvp_migrations. A slice adds its
//   own new file (a new table, or ALTER TABLE for a column) and never edits
//   another slice's or one that has shipped: a recorded file doesn't run again.
// - Caches that aren't the game's record stay out of MvpStore (slice 10's name
//   cache lives in ./fusions.ts).
import type { BattleRecord, Champion, DayState, FightKind, FusionDiscovery, Ghost, PlayerRef, PlayoffResult, Rating, Slay, UnitId } from "../../../src/mvp/contract.js";
import type { MvpRunState } from "../../../src/mvp/run.js";

export interface MvpStore {
  addPlayer(p: PlayerRef): void;
  player(id: string): PlayerRef | undefined;
  putRun(r: MvpRunState): void;
  run(id: string): MvpRunState | undefined;
  activeRun(playerId: string): MvpRunState | undefined;
  addGhost(g: Ghost): void;
  /** Saved teams at this round built with `contentVersion`, never one of
   * `excludePlayerId`'s own, oldest first; with `limit`, only the newest
   * `limit` of them. */
  ghosts(round: number, opts: { excludePlayerId: string; contentVersion: string; limit?: number }): Ghost[];
  putBattle(b: BattleRecord): void;
  battle(id: string): BattleRecord | undefined;
  /** Stored battles, oldest first: of one kind, and/or fought at or after
   * `since` (ISO). Slice 11's win and pick rates, slice 5's day. */
  battles(opts?: { kind?: FightKind; since?: string }): BattleRecord[];
  // Discovered fusions, one per ordered pair. Slice 10 owns them and is the
  // only writer (the credit rule is on FusionDiscovery); everyone else reads.
  fusion(first: UnitId, second: UnitId): FusionDiscovery | undefined;
  /** Stores or replaces the pair's discovery (replacing is how a human claims a bot's pair). */
  putFusion(f: FusionDiscovery): void;
  fusions(): FusionDiscovery[];
  // The day (contract.ts, "day, champion, rating", says who writes what).
  /** The latest day (highest seq); read it through RunDeps.today(). */
  currentDay(): DayState | undefined;
  /** Stores or replaces the day with this seq (slice 5; day.ts creates day 1). */
  putDay(d: DayState): void;
  champion(seq: number): Champion | undefined;
  /** The champion of the latest day (highest seq). */
  currentChampion(): Champion | undefined;
  putChampion(c: Champion): void;
  /** Every day's champion, oldest first. */
  champions(): Champion[];
  addSlay(s: Slay): void;
  slays(seq: number): Slay[];
  /** The playoff played by day `seq`'s slayers (slice 5 writes it at rollover). */
  playoff(seq: number): PlayoffResult | undefined;
  putPlayoff(p: PlayoffResult): void;
  rating(playerId: string): Rating | undefined;
  putRating(r: Rating): void;
  // Slice 11's unit tallies, per content version; only ./stats.ts writes them.
  /** Adds `delta` to `contentVersion`'s tallies (runs and each unit's counts). */
  addUnitTallies(contentVersion: string, delta: UnitTallies): void;
  /** The running totals for `contentVersion`; zero runs and no units before any. */
  unitTallies(contentVersion: string): UnitTallies;
  // Slice 13's invite links and sessions; only ./invites.ts writes them.
  /** Players with this name (by `nameKey`): humans only, or bots too with `bots`. */
  playersNamed(name: string, opts?: { bots?: boolean }): PlayerRef[];
  /** Stores or replaces the invite with this code. */
  putInvite(i: Invite): void;
  invite(code: string): Invite | undefined;
  invites(): Invite[];
  /** `tokenHash`: the hex SHA-256 of a session token; the token itself is never stored. */
  addSession(tokenHash: string, playerId: string, at: string): void;
  /** The player id of the session with this token hash. */
  sessionPlayer(tokenHash: string): string | undefined;
  /** Ends every session of this player (a revoked link); returns how many. */
  deleteSessions(playerId: string): number;
  /** Swaps the invite `oldCode` for `next` (a rotated code) in one step. */
  replaceInvite(oldCode: string, next: Invite): void;
}

/** How names compare: Unicode-folded (NFKC) and lower-cased in JS, so
 * Cyrillic and full-width letters match too (SQLite's lower() is ASCII only). */
export const nameKey = (name: string) => name.normalize("NFKC").toLowerCase();

/** One person's invite link (slice 13). Its code is the secret in the URL;
 * opening it on any device gives the same player. Names are unique among
 * invites (case-insensitive). `admin` may use the dev tools. */
export interface Invite {
  code: string;
  name: string;
  playerId: string;
  admin: boolean;
  createdAt: string;
  /** When it was first opened. */
  redeemedAt: string | null;
}

/** Counted as runs go (./stats.ts): `runs` is the finished runs, and per unit
 * the fights its team fought and won and the finished runs it ended on the
 * line in. A fused unit counts for both parts. */
export interface UnitTallies {
  runs: number;
  units: UnitTally[];
}

export interface UnitTally {
  unitId: UnitId;
  fights: number;
  wins: number;
  runs: number;
}

export class MemoryMvpStore implements MvpStore {
  private players = new Map<string, PlayerRef>();
  private runs = new Map<string, MvpRunState>();
  private ghostsByRound = new Map<number, Ghost[]>();
  private battlesById = new Map<string, BattleRecord>();
  private fusionsByPair = new Map<string, FusionDiscovery>();
  private championsBySeq = new Map<number, Champion>();
  private slaysBySeq = new Map<number, Slay[]>();
  private ratings = new Map<string, Rating>();
  private daysBySeq = new Map<number, DayState>();
  private playoffsBySeq = new Map<number, PlayoffResult>();
  private talliesByVersion = new Map<string, { runs: number; units: Map<UnitId, UnitTally> }>();
  private invitesByCode = new Map<string, Invite>();
  private sessions = new Map<string, string>();
  addPlayer(p: PlayerRef): void { this.players.set(p.id, p); }
  player(id: string): PlayerRef | undefined { return this.players.get(id); }
  putRun(r: MvpRunState): void { this.runs.set(r.runId, r); }
  run(id: string): MvpRunState | undefined { return this.runs.get(id); }
  activeRun(playerId: string): MvpRunState | undefined {
    for (const r of this.runs.values()) if (r.player.id === playerId && r.phase !== "over") return r;
    return undefined;
  }
  addGhost(g: Ghost): void {
    const list = this.ghostsByRound.get(g.round) ?? [];
    list.push(g);
    this.ghostsByRound.set(g.round, list);
  }
  ghosts(round: number, opts: { excludePlayerId: string; contentVersion: string; limit?: number }): Ghost[] {
    const all = (this.ghostsByRound.get(round) ?? []).filter((g) => g.player.id !== opts.excludePlayerId && g.contentVersion === opts.contentVersion);
    return opts.limit === undefined ? all : all.slice(Math.max(0, all.length - opts.limit));
  }
  putBattle(b: BattleRecord): void { this.battlesById.set(b.battleId, b); }
  battle(id: string): BattleRecord | undefined { return this.battlesById.get(id); }
  battles(opts: { kind?: FightKind; since?: string } = {}): BattleRecord[] {
    return [...this.battlesById.values()]
      .filter((b) => (opts.kind === undefined || b.kind === opts.kind) && (opts.since === undefined || b.at >= opts.since))
      .sort((a, b) => (a.at < b.at ? -1 : a.at > b.at ? 1 : 0));
  }
  fusion(first: UnitId, second: UnitId): FusionDiscovery | undefined { return this.fusionsByPair.get(pairKey(first, second)); }
  putFusion(f: FusionDiscovery): void { this.fusionsByPair.set(pairKey(f.first, f.second), f); }
  fusions(): FusionDiscovery[] { return [...this.fusionsByPair.values()]; }
  champion(seq: number): Champion | undefined { return this.championsBySeq.get(seq); }
  currentChampion(): Champion | undefined { return this.champions().at(-1); }
  putChampion(c: Champion): void { this.championsBySeq.set(c.seq, c); }
  champions(): Champion[] { return [...this.championsBySeq.values()].sort((a, b) => a.seq - b.seq); }
  addSlay(s: Slay): void {
    const list = this.slaysBySeq.get(s.seq) ?? [];
    list.push(s);
    this.slaysBySeq.set(s.seq, list);
  }
  slays(seq: number): Slay[] { return [...(this.slaysBySeq.get(seq) ?? [])]; }
  playoff(seq: number): PlayoffResult | undefined { return this.playoffsBySeq.get(seq); }
  putPlayoff(p: PlayoffResult): void { this.playoffsBySeq.set(p.seq, p); }
  currentDay(): DayState | undefined {
    let last: DayState | undefined;
    for (const d of this.daysBySeq.values()) if (!last || d.seq > last.seq) last = d;
    return last;
  }
  putDay(d: DayState): void { this.daysBySeq.set(d.seq, d); }
  rating(playerId: string): Rating | undefined { return this.ratings.get(playerId); }
  putRating(r: Rating): void { this.ratings.set(r.player.id, r); }
  addUnitTallies(contentVersion: string, delta: UnitTallies): void {
    const t = this.talliesByVersion.get(contentVersion) ?? { runs: 0, units: new Map<UnitId, UnitTally>() };
    t.runs += delta.runs;
    for (const d of delta.units) {
      const u = t.units.get(d.unitId) ?? { unitId: d.unitId, fights: 0, wins: 0, runs: 0 };
      t.units.set(d.unitId, { unitId: d.unitId, fights: u.fights + d.fights, wins: u.wins + d.wins, runs: u.runs + d.runs });
    }
    this.talliesByVersion.set(contentVersion, t);
  }
  unitTallies(contentVersion: string): UnitTallies {
    const t = this.talliesByVersion.get(contentVersion);
    return { runs: t?.runs ?? 0, units: t ? [...t.units.values()].map((u) => ({ ...u })) : [] };
  }
  playersNamed(name: string, opts: { bots?: boolean } = {}): PlayerRef[] {
    const key = nameKey(name);
    return [...this.players.values()].filter((p) => (opts.bots || !p.bot) && nameKey(p.name) === key);
  }
  putInvite(i: Invite): void {
    for (const other of this.invitesByCode.values())
      if (other.code !== i.code && nameKey(other.name) === nameKey(i.name)) throw new Error(`an invite named ${i.name} exists`);
    this.invitesByCode.set(i.code, { ...i });
  }
  replaceInvite(oldCode: string, next: Invite): void {
    const old = this.invitesByCode.get(oldCode);
    this.invitesByCode.delete(oldCode);
    try {
      this.putInvite(next);
    } catch (e) {
      if (old) this.invitesByCode.set(oldCode, old);
      throw e;
    }
  }
  invite(code: string): Invite | undefined { const i = this.invitesByCode.get(code); return i && { ...i }; }
  invites(): Invite[] { return [...this.invitesByCode.values()].map((i) => ({ ...i })); }
  addSession(tokenHash: string, playerId: string): void { this.sessions.set(tokenHash, playerId); }
  sessionPlayer(tokenHash: string): string | undefined { return this.sessions.get(tokenHash); }
  deleteSessions(playerId: string): number {
    let n = 0;
    for (const [h, id] of this.sessions) if (id === playerId && this.sessions.delete(h)) n++;
    return n;
  }
}

/** Ordered: (a, b) and (b, a) are different fusions. */
function pairKey(first: UnitId, second: UnitId): string {
  return JSON.stringify([first, second]);
}
