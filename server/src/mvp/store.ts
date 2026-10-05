// MVP storage seam (mission #574). Slice 1 keeps everything in memory behind
// this interface; slice 4 swaps in SQLite without touching the routes.
import type { BattleRecord, Champion, FusionDiscovery, Ghost, PlayerRef, Rating, Slay, UnitId } from "../../../src/mvp/contract.js";
import type { MvpRunState } from "../../../src/mvp/run.js";

export interface MvpStore {
  addPlayer(p: PlayerRef): void;
  player(id: string): PlayerRef | undefined;
  putRun(r: MvpRunState): void;
  run(id: string): MvpRunState | undefined;
  activeRun(playerId: string): MvpRunState | undefined;
  addGhost(g: Ghost): void;
  /** Saved teams at this round, from runs other than `excludeRunId`. */
  ghosts(round: number, excludeRunId: string): Ghost[];
  putBattle(b: BattleRecord): void;
  battle(id: string): BattleRecord | undefined;
  // Discovered fusions, one per ordered pair. Slice 10 owns them and is the
  // only writer (the credit rule is on FusionDiscovery); everyone else reads.
  fusion(first: UnitId, second: UnitId): FusionDiscovery | undefined;
  /** Stores or replaces the pair's discovery (replacing is how a human claims a bot's pair). */
  putFusion(f: FusionDiscovery): void;
  fusions(): FusionDiscovery[];
  // The day (contract.ts, "day, champion, rating"): slice 6 seeds day 1 with
  // putChampion when champions() is empty; slice 5 owns every later
  // putChampion and the rollover; slice 4's Crown fight writes the Slay and
  // the run-end rating.
  champion(seq: number): Champion | undefined;
  /** The champion of the latest day (highest seq). */
  currentChampion(): Champion | undefined;
  putChampion(c: Champion): void;
  /** Every day's champion, oldest first. */
  champions(): Champion[];
  addSlay(s: Slay): void;
  slays(seq: number): Slay[];
  rating(playerId: string): Rating | undefined;
  putRating(r: Rating): void;
}

export class MemoryMvpStore implements MvpStore {
  private players = new Map<string, PlayerRef>();
  private runs = new Map<string, MvpRunState>();
  private ghostsByRound = new Map<number, Ghost[]>();
  private battles = new Map<string, BattleRecord>();
  private fusionsByPair = new Map<string, FusionDiscovery>();
  private championsBySeq = new Map<number, Champion>();
  private slaysBySeq = new Map<number, Slay[]>();
  private ratings = new Map<string, Rating>();
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
  ghosts(round: number, excludeRunId: string): Ghost[] {
    return (this.ghostsByRound.get(round) ?? []).filter((g) => g.runId !== excludeRunId);
  }
  putBattle(b: BattleRecord): void { this.battles.set(b.battleId, b); }
  battle(id: string): BattleRecord | undefined { return this.battles.get(id); }
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
  rating(playerId: string): Rating | undefined { return this.ratings.get(playerId); }
  putRating(r: Rating): void { this.ratings.set(r.player.id, r); }
}

/** Ordered: (a, b) and (b, a) are different fusions. */
function pairKey(first: UnitId, second: UnitId): string {
  return JSON.stringify([first, second]);
}
