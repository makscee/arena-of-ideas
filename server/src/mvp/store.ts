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
import type { BattleRecord, Champion, DayState, FightKind, FusionDiscovery, Ghost, Idea, IdeaState, PlayerRef, PlayoffResult, Rating, Slay, UnitId } from "../../../src/mvp/contract.js";
import type { MvpRunState } from "../../../src/mvp/run.js";
import type { Row } from "../../../src/mvp/units.js";

export interface MvpStore {
  addPlayer(p: PlayerRef): void;
  player(id: string): PlayerRef | undefined;
  putRun(r: MvpRunState): void;
  run(id: string): MvpRunState | undefined;
  activeRun(playerId: string): MvpRunState | undefined;
  addGhost(g: Ghost): void;
  /** Saved teams at this round, made on any pool (M2-2), never one of
   * `excludePlayerId`'s own, oldest first; with `limit`, only the newest
   * `limit` of them. */
  ghosts(round: number, opts: { excludePlayerId: string; limit?: number }): Ghost[];
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
  /** Opens the invite `code` in one step: a session (`tokenHash`, the hex
   * SHA-256 of its token; the token itself is never stored) for its player,
   * only if the code still exists when the session is written, so a revoke
   * can't race it. Marks the invite opened, keeps the player's newest
   * `MAX_SESSIONS` sessions, and returns the player id (undefined: no such code). */
  redeemInvite(code: string, tokenHash: string, at: string): string | undefined;
  /** The player id of the session with this token hash. */
  sessionPlayer(tokenHash: string): string | undefined;
  /** Swaps the invite `oldCode` for `next` (a rotated code) and ends every
   * session of its player, in one step; returns how many sessions ended. */
  rotateInvite(oldCode: string, next: Invite): number;
  // R4-20's open join link (#704); only ./invites.ts writes it.
  /** The shared join code, or undefined before `mvp:invite -- open` made one. */
  joinCode(): string | undefined;
  /** Sets (or rotates) the shared join code. */
  setJoinCode(code: string): void;
  // M4-6's Telegram login (mission #810); only ./telegram.ts writes it.
  /** Adds a session for this player (a Telegram login), keeping the player's
   * newest `MAX_SESSIONS`, as opening an invite link does. */
  addSession(playerId: string, tokenHash: string, at: string): void;
  /** Stores or replaces a login code (by its hash). */
  putTelegramCode(c: TelegramCode): void;
  telegramCode(codeHash: string): TelegramCode | undefined;
  /** Removes a login code; true when it was there (so only one caller gets it). */
  dropTelegramCode(codeHash: string): boolean;
  /** Links a Telegram user and a player. Throws when either is linked already. */
  linkTelegram(l: TelegramLink): void;
  /** The link of this Telegram user, or of this player. */
  telegramLink(by: { tgUserId: string } | { playerId: string }): TelegramLink | undefined;
  /** Removes this player's link; true when there was one. */
  unlinkTelegram(playerId: string): boolean;
  // M2-1's units as data (mission #735); ./pool.ts seeds and reads them.
  /** Stores or replaces the unit with this id. */
  putUnit(u: StoredUnit): void;
  unit(id: UnitId): StoredUnit | undefined;
  /** Every unit (or those with `status`), oldest first. */
  units(opts?: { status?: UnitStatus }): StoredUnit[];
  /** Adds a pool snapshot; it becomes the current one. */
  putPool(p: PoolSnapshot): void;
  /** The newest snapshot with this content version. */
  pool(version: string): PoolSnapshot | undefined;
  /** The newest snapshot: the live pool. */
  currentPool(): PoolSnapshot | undefined;
  /** Stores or replaces the stint (unit, enteredSeq). */
  putStint(s: PoolStint): void;
  /** Every stint (or one unit's), by unit then enteredSeq. */
  stints(unitId?: UnitId): PoolStint[];
  /** Writes the first pool in one step: the units, the snapshot and the
   * stints, only while there is no pool yet. False: there was one, nothing written. */
  seedPool(units: StoredUnit[], pool: PoolSnapshot, stints: PoolStint[]): boolean;
  /** Runs `fn` as one write: on SQLite one IMMEDIATE transaction, so a crash
   * or a throw midway leaves nothing of it (M2-2's sync-seed and swap). */
  atomically<T>(fn: () => T): T;
  /** Adds `delta` to day `daySeq`'s tallies (only ./stats.ts writes them). */
  addDayTallies(daySeq: number, delta: DayTallies): void;
  /** Day `daySeq`'s running totals; zero runs and no units before any. */
  dayTallies(daySeq: number): DayTallies;
  // Mission 2's idea counts (M2-3); only ./ideas.ts writes them.
  /** The player's counts; all zero before any write. */
  ideaCounts(playerId: string): IdeaCounts;
  putIdeaCounts(playerId: string, c: IdeaCounts): void;
  // Mission 2's written ideas (M2-4); ./ideas.ts and the later stages write them.
  /** Stores or replaces the idea with this id. */
  putIdea(i: Idea): void;
  idea(ideaId: string): Idea | undefined;
  /** Ideas oldest first: one player's, and/or in one state (the reader's
   * queue), and/or the evolve proposals for one Library unit (M3-3, `target`). */
  ideas(opts?: { playerId?: string; state?: IdeaState; target?: UnitId }): Idea[];
  /** Removes an idea (a `written` one its author took back). */
  deleteIdea(ideaId: string): void;
  // Mission 2's new-words log (M2-5): what ideas needed that the game can't
  // say yet. Only additions; ./idea-reading.ts writes it.
  addWordRequest(w: WordRequest): void;
  /** Every request, oldest first. */
  wordRequests(): WordRequest[];
  // Mission 2's votes (M2-8); only ./votes.ts writes them.
  /** Adds a vote; false (nothing written) when the player already voted on this pair. */
  addVote(v: Vote): boolean;
  /** Votes oldest first: on one candidate, and/or by one player. */
  votes(opts?: { candidateId?: UnitId; playerId?: string }): Vote[];
  // Mission 4's daily post to Telegram (M4-8); only ./daily-post.ts writes it.
  /** Claims day `seq`'s message `key` ("ru", "en", "ru+en") as being sent;
   * false (nothing written) when it was claimed before: a day posts once. */
  claimDailyPost(seq: number, key: string, at: string): boolean;
  /** Records how a claimed message went. */
  putDailyPost(p: DailyPost): void;
  /** The messages claimed for one day, or every day's, in claim order. */
  dailyPosts(seq?: number): DailyPost[];
}

/** One message of a day's post (M4-8): claimed ("sending") before it is
 * sent, then "sent" or "failed" after its last try. */
export interface DailyPost {
  seq: number;
  key: string;
  state: "sending" | "sent" | "failed";
  tries: number;
  at: string;
  error?: string;
}

/** An idea the reader couldn't fully make (M2-5): the game word it lacks
 * ("steal gold"), as the model named it, and the part of the idea's text
 * that needed it. */
export interface WordRequest {
  ideaId: string;
  word: string;
  part: string;
  createdAt: string;
}

/** One player's either/or between a candidate and a live unit (M2-8):
 * `pick` is the unit chosen, null for a skip. One per player per pair. */
export interface Vote {
  playerId: string;
  candidateId: UnitId;
  otherId: UnitId;
  pick: UnitId | null;
  createdAt: string;
}

export type UnitStatus = "candidate" | "live" | "library" | "rejected";
export type UnitOrigin = "idea" | "evolution" | "return" | "seed";

/** A unit as data (M2-1). Its id is permanent and never reused, even for a
 * unit with the same name; `row` is what the pool is built from (mvpPool). */
export interface StoredUnit {
  unitId: UnitId;
  status: UnitStatus;
  row: Row;
  /** The player whose idea it was; null for the seed. */
  authorId: string | null;
  origin: UnitOrigin;
  /** The unit it evolved from or returns as; null otherwise. */
  parentId: UnitId | null;
  /** M3-3: the archetype's first version (./lineage.ts). Absent: the unit is
   * its own root, as every unit made before mission 3 is. */
  rootId?: UnitId;
  createdAt: string;
}

/** The live units at a day: `version` is the content version built from
 * `unitIds`, in this order. */
export interface PoolSnapshot {
  version: string;
  daySeq: number;
  unitIds: UnitId[];
  createdAt: string;
  /** The units' rows as they were in this pool (M2-2), so a run pinned to it
   * buys what it bought before a later change to a unit's row. Missing on the
   * seed snapshot: its rows are the stored units'. */
  rows?: Row[];
}

/** Why a stint began or ended: the seed pool, an idea's or an evolution's
 * entrant, a return of a Library unit unchanged (M3-3), the rotation's
 * leaver, a dev swap or credit. */
export type StintReason = "seed" | "idea" | "evolution" | "return" | "rotated" | "swapped" | "dev: credited";

/** One stay of a unit in the live pool, from day `enteredSeq` to `leftSeq`
 * (null: still there). */
export interface PoolStint {
  unitId: UnitId;
  enteredSeq: number;
  leftSeq: number | null;
  /** Why it entered (while there) or left. */
  reason: StintReason;
}

/** Day tallies: finished runs, and per unit what UnitTally counts plus picks. */
export interface DayTallies {
  runs: number;
  units: UnitDayTally[];
}

export interface UnitDayTally extends UnitTally {
  /** Times it was bought from the shop or taken as a gift. */
  picks: number;
}

/** What a player's ideas are counted from, besides their finished runs
 * (Rating.runs): ./ideas.ts derives the ideas held from both. */
export interface IdeaCounts {
  /** Ideas written (M2-4) or refunded back (a negative step). */
  spent: number;
  /** Dev "+1 idea". */
  granted: number;
  /** Ideas earned while holding the cap, so never held. */
  forfeited: number;
}

/** Devices per player: opening a link past this ends the oldest session. */
export const MAX_SESSIONS = 10;

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

/** A Telegram login code (M4-6), kept by the SHA-256 of the code in the deep
 * link. `playerId`: the player who asked (Link Telegram), or null (Log in).
 * `asked`: what the bot asked the Telegram user who pressed Start; their Yes
 * sets `readyFor` to the player the device logs in as (the page's next poll
 * takes it, once), their No sets `declined`. */
export interface TelegramCode {
  codeHash: string;
  playerId: string | null;
  createdAt: string;
  expiresAt: string;
  readyFor: string | null;
  asked?: TelegramAsk;
  declined?: boolean;
}

/** What a code does for a Telegram user, worked out when they press Start and
 * again when they tap Yes (it must not have changed in between): log a device
 * in as their linked player, link the asking player, or make a new player. */
export type TelegramAsk = { kind: "login" | "link"; playerId: string; tgUserId: string } | { kind: "new"; playerId: null; tgUserId: string };

/** One Telegram user linked to one player (M4-6). */
export interface TelegramLink {
  tgUserId: string;
  playerId: string;
  linkedAt: string;
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
  /** token hash → player id, oldest first (Map keeps insertion order). */
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
  ghosts(round: number, opts: { excludePlayerId: string; limit?: number }): Ghost[] {
    const all = (this.ghostsByRound.get(round) ?? []).filter((g) => g.player.id !== opts.excludePlayerId);
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
  rotateInvite(oldCode: string, next: Invite): number {
    const old = this.invitesByCode.get(oldCode);
    this.invitesByCode.delete(oldCode);
    try {
      this.putInvite(next);
    } catch (e) {
      if (old) this.invitesByCode.set(oldCode, old);
      throw e;
    }
    let n = 0;
    for (const [h, id] of this.sessions) if (id === next.playerId && this.sessions.delete(h)) n++;
    return n;
  }
  invite(code: string): Invite | undefined { const i = this.invitesByCode.get(code); return i && { ...i }; }
  invites(): Invite[] { return [...this.invitesByCode.values()].map((i) => ({ ...i })); }
  redeemInvite(code: string, tokenHash: string, at: string): string | undefined {
    const i = this.invitesByCode.get(code);
    if (!i) return undefined;
    this.sessions.set(tokenHash, i.playerId);
    i.redeemedAt ??= at;
    const mine = [...this.sessions].filter(([, id]) => id === i.playerId);
    for (const [h] of mine.slice(0, Math.max(0, mine.length - MAX_SESSIONS))) this.sessions.delete(h);
    return i.playerId;
  }
  sessionPlayer(tokenHash: string): string | undefined { return this.sessions.get(tokenHash); }
  private join: string | undefined;
  joinCode(): string | undefined { return this.join; }
  setJoinCode(code: string): void { this.join = code; }
  addSession(playerId: string, tokenHash: string, _at: string): void {
    this.sessions.set(tokenHash, playerId);
    const mine = [...this.sessions].filter(([, id]) => id === playerId);
    for (const [h] of mine.slice(0, Math.max(0, mine.length - MAX_SESSIONS))) this.sessions.delete(h);
  }
  private tgCodes = new Map<string, TelegramCode>();
  private tgLinks = new Map<string, TelegramLink>();
  putTelegramCode(c: TelegramCode): void { this.tgCodes.set(c.codeHash, { ...c }); }
  telegramCode(codeHash: string): TelegramCode | undefined { const c = this.tgCodes.get(codeHash); return c && { ...c }; }
  dropTelegramCode(codeHash: string): boolean { return this.tgCodes.delete(codeHash); }
  linkTelegram(l: TelegramLink): void {
    if (this.tgLinks.has(l.tgUserId) || this.telegramLink({ playerId: l.playerId })) throw new Error(`telegram ${l.tgUserId} or player ${l.playerId} is linked already`);
    this.tgLinks.set(l.tgUserId, { ...l });
  }
  telegramLink(by: { tgUserId: string } | { playerId: string }): TelegramLink | undefined {
    const l = "tgUserId" in by ? this.tgLinks.get(by.tgUserId) : [...this.tgLinks.values()].find((x) => x.playerId === by.playerId);
    return l && { ...l };
  }
  unlinkTelegram(playerId: string): boolean {
    const l = this.telegramLink({ playerId });
    return !!l && this.tgLinks.delete(l.tgUserId);
  }
  private unitsById = new Map<UnitId, StoredUnit>();
  private pools: PoolSnapshot[] = [];
  private stintsByKey = new Map<string, PoolStint>();
  private dayTalliesBySeq = new Map<number, { runs: number; units: Map<UnitId, UnitDayTally> }>();
  putUnit(u: StoredUnit): void { this.unitsById.set(u.unitId, structuredClone(u)); }
  unit(id: UnitId): StoredUnit | undefined { const u = this.unitsById.get(id); return u && structuredClone(u); }
  units(opts: { status?: UnitStatus } = {}): StoredUnit[] {
    return [...this.unitsById.values()].filter((u) => opts.status === undefined || u.status === opts.status).map((u) => structuredClone(u));
  }
  putPool(p: PoolSnapshot): void { this.pools.push(structuredClone(p)); }
  pool(version: string): PoolSnapshot | undefined { const p = [...this.pools].reverse().find((x) => x.version === version); return p && structuredClone(p); }
  currentPool(): PoolSnapshot | undefined { const p = this.pools.at(-1); return p && structuredClone(p); }
  putStint(s: PoolStint): void { this.stintsByKey.set(JSON.stringify([s.unitId, s.enteredSeq]), { ...s }); }
  stints(unitId?: UnitId): PoolStint[] {
    return [...this.stintsByKey.values()]
      .filter((s) => unitId === undefined || s.unitId === unitId)
      .sort((a, b) => (a.unitId < b.unitId ? -1 : a.unitId > b.unitId ? 1 : a.enteredSeq - b.enteredSeq))
      .map((s) => ({ ...s }));
  }
  seedPool(units: StoredUnit[], pool: PoolSnapshot, stints: PoolStint[]): boolean {
    if (this.pools.length) return false;
    for (const u of units) this.putUnit(u);
    this.putPool(pool);
    for (const s of stints) this.putStint(s);
    return true;
  }
  atomically<T>(fn: () => T): T { return fn(); }
  addDayTallies(daySeq: number, delta: DayTallies): void {
    const t = this.dayTalliesBySeq.get(daySeq) ?? { runs: 0, units: new Map<UnitId, UnitDayTally>() };
    t.runs += delta.runs;
    for (const d of delta.units) {
      const u = t.units.get(d.unitId) ?? { unitId: d.unitId, fights: 0, wins: 0, runs: 0, picks: 0 };
      t.units.set(d.unitId, { unitId: d.unitId, fights: u.fights + d.fights, wins: u.wins + d.wins, runs: u.runs + d.runs, picks: u.picks + d.picks });
    }
    this.dayTalliesBySeq.set(daySeq, t);
  }
  dayTallies(daySeq: number): DayTallies {
    const t = this.dayTalliesBySeq.get(daySeq);
    return { runs: t?.runs ?? 0, units: t ? [...t.units.values()].sort((a, b) => (a.unitId < b.unitId ? -1 : a.unitId > b.unitId ? 1 : 0)).map((u) => ({ ...u })) : [] };
  }
  private ideaCountsBy = new Map<string, IdeaCounts>();
  ideaCounts(playerId: string): IdeaCounts { return { ...(this.ideaCountsBy.get(playerId) ?? NO_IDEAS) }; }
  putIdeaCounts(playerId: string, c: IdeaCounts): void { this.ideaCountsBy.set(playerId, { ...c }); }
  private written = new Map<string, Idea>();
  putIdea(i: Idea): void {
    this.written.set(i.ideaId, structuredClone(i));
  }
  idea(ideaId: string): Idea | undefined { const i = this.written.get(ideaId); return i && structuredClone(i); }
  ideas(opts: { playerId?: string; state?: IdeaState; target?: UnitId } = {}): Idea[] {
    return [...this.written.values()]
      .filter((i) => (opts.playerId === undefined || i.playerId === opts.playerId) && (opts.state === undefined || i.state === opts.state) && (opts.target === undefined || i.data.target === opts.target))
      .map((i) => structuredClone(i));
  }
  deleteIdea(ideaId: string): void { this.written.delete(ideaId); }
  private votesList: Vote[] = [];
  addVote(v: Vote): boolean {
    if (this.votesList.some((x) => x.playerId === v.playerId && x.candidateId === v.candidateId && x.otherId === v.otherId)) return false;
    this.votesList.push({ ...v });
    return true;
  }
  votes(opts: { candidateId?: UnitId; playerId?: string } = {}): Vote[] {
    return this.votesList.filter((v) => (opts.candidateId === undefined || v.candidateId === opts.candidateId) && (opts.playerId === undefined || v.playerId === opts.playerId)).map((v) => ({ ...v }));
  }
  private words: WordRequest[] = [];
  addWordRequest(w: WordRequest): void { this.words.push({ ...w }); }
  wordRequests(): WordRequest[] { return this.words.map((w) => ({ ...w })); }
  private posts: DailyPost[] = [];
  claimDailyPost(seq: number, key: string, at: string): boolean {
    if (this.posts.some((p) => p.seq === seq && p.key === key)) return false;
    this.posts.push({ seq, key, state: "sending", tries: 0, at });
    return true;
  }
  putDailyPost(p: DailyPost): void {
    const i = this.posts.findIndex((x) => x.seq === p.seq && x.key === p.key);
    if (i >= 0) this.posts[i] = { ...p };
    else this.posts.push({ ...p });
  }
  dailyPosts(seq?: number): DailyPost[] { return this.posts.filter((p) => seq === undefined || p.seq === seq).map((p) => ({ ...p })); }
}

export const NO_IDEAS: IdeaCounts = { spent: 0, granted: 0, forfeited: 0 };

/** Ordered: (a, b) and (b, a) are different fusions. */
function pairKey(first: UnitId, second: UnitId): string {
  return JSON.stringify([first, second]);
}
